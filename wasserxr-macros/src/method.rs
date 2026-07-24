use quote::{format_ident, quote};
use syn::{
    Error, FnArg, ItemFn, Pat, Path, Result, ReturnType, Type,
    ext::IdentExt,
    parse::{Parse, ParseStream},
    spanned::Spanned,
};

pub(crate) struct Args {
    component_type: Path,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        let component_type = input.parse()?;
        if !input.is_empty() {
            return Err(input.error("`method` only supports one component type"));
        }

        Ok(Self { component_type })
    }
}

struct Argument {
    ident: syn::Ident,
    name: String,
    ty: Type,
}

pub(crate) fn expand(args: Args, item: ItemFn) -> Result<proc_macro2::TokenStream> {
    let component_type = &args.component_type;
    let component_id = component_type
        .segments
        .last()
        .ok_or_else(|| Error::new_spanned(component_type, "component type path cannot be empty"))?
        .ident
        .to_string();

    let function_name = item.sig.ident.clone();
    let method_name = function_name.to_string();

    let mut inputs = item.sig.inputs.iter();

    let scene = inputs.next().ok_or_else(|| {
        Error::new_spanned(&item.sig, "`method` requires a `&mut Scene` parameter")
    })?;
    expect_mut_reference_named(
        scene,
        "Scene",
        "the first parameter must be exactly `&mut Scene`",
    )?;

    let component = inputs.next().ok_or_else(|| {
        Error::new_spanned(&item.sig, "`method` requires a mutable component reference")
    })?;
    let component_ident = component_type
        .segments
        .last()
        .expect("checked above")
        .ident
        .to_string();
    expect_mut_reference_named(
        component,
        &component_ident,
        "the second parameter must be a mutable reference to the component type",
    )?;

    let mut arguments = Vec::new();
    for input in inputs {
        arguments.push(parse_argument(input)?);
    }

    check_return_type(&item.sig.output)?;

    let symbol = format!("wxr_method_{}_{}", component_id, method_name);
    let wrapper_ident = format_ident!("{}", symbol);

    let resolvers = arguments.iter().map(|argument| {
        let ident = &argument.ident;
        let ty = &argument.ty;
        let name = std::ffi::CString::new(argument.name.as_str())
            .expect("argument name cannot contain a null byte");
        let name = proc_macro2::Literal::c_string(&name);

        quote! {
            let #ident: &mut #ty =
                match ::wasserxr::scene::component::methods::find_argument::<#ty>(
                    __arguments,
                    #name,
                ) {
                    Ok(__reference) => __reference,
                    Err(__status) => {
                        return ::wasserxr::scene::component::methods::WXRMethodResult {
                            status: __status,
                            action_error: 0,
                            value: ::std::ptr::null_mut(),
                        };
                    }
                };
        }
    });

    let argument_idents = arguments.iter().map(|argument| &argument.ident);

    Ok(quote! {
        #item

        #[doc = "Generated WasserXR component method binding."]
        #[doc = ""]
        #[doc = "Resolves named arguments, casts them unchecked, and calls the annotated method."]
        #[doc = ""]
        #[doc = "# Safety"]
        #[doc = ""]
        #[doc = "`scene` and `component` must be valid pointers for this component type, and every argument pointer must point to the type declared by the matching parameter."]
        #[unsafe(export_name = #symbol)]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn #wrapper_ident(
            scene: *mut ::wasserxr::bindings::scene::WXRScene,
            component: *mut ::std::ffi::c_void,
            arguments: *const ::wasserxr::scene::component::methods::WXRArgument,
            argument_count: usize,
        ) -> ::wasserxr::scene::component::methods::WXRMethodResult {
            let __arguments: &[::wasserxr::scene::component::methods::WXRArgument] =
                if arguments.is_null() {
                    &[]
                } else {
                    unsafe { ::std::slice::from_raw_parts(arguments, argument_count) }
                };

            #(#resolvers)*

            let __scene = unsafe { &mut *(scene as *mut ::wasserxr::scene::Scene) };
            let __component = unsafe { &mut *(component as *mut #component_type) };

            match #function_name(__scene, __component, #(#argument_idents),*) {
                Ok(__value) => ::wasserxr::scene::component::methods::WXRMethodResult {
                    status: ::wasserxr::scene::component::methods::WXRMethodStatus::Success,
                    action_error: 0,
                    value: __value,
                },
                Err(__code) => ::wasserxr::scene::component::methods::WXRMethodResult {
                    status: ::wasserxr::scene::component::methods::WXRMethodStatus::ActionError,
                    action_error: __code,
                    value: ::std::ptr::null_mut(),
                },
            }
        }
    })
}

fn expect_mut_reference_named(input: &FnArg, name: &str, message: &str) -> Result<()> {
    let FnArg::Typed(pat_type) = input else {
        return Err(Error::new_spanned(input, message));
    };

    let Type::Reference(reference) = pat_type.ty.as_ref() else {
        return Err(Error::new_spanned(input, message));
    };

    if reference.mutability.is_none() {
        return Err(Error::new_spanned(input, message));
    }

    let Type::Path(path) = reference.elem.as_ref() else {
        return Err(Error::new_spanned(input, message));
    };

    let matches = path
        .path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == name);
    if !matches {
        return Err(Error::new_spanned(input, message));
    }

    Ok(())
}

fn parse_argument(input: &FnArg) -> Result<Argument> {
    let message = "method arguments must be `&mut T` with a simple identifier name";

    let FnArg::Typed(pat_type) = input else {
        return Err(Error::new_spanned(input, message));
    };

    let Pat::Ident(pat_ident) = pat_type.pat.as_ref() else {
        return Err(Error::new_spanned(input, message));
    };

    if pat_ident.subpat.is_some() || pat_ident.by_ref.is_some() {
        return Err(Error::new_spanned(input, message));
    }

    let Type::Reference(reference) = pat_type.ty.as_ref() else {
        return Err(Error::new_spanned(input, message));
    };

    if reference.mutability.is_none() {
        return Err(Error::new_spanned(input, message));
    }

    let ident = pat_ident.ident.clone();
    let name = ident.unraw().to_string();

    Ok(Argument {
        ident,
        name,
        ty: reference.elem.as_ref().clone(),
    })
}

fn check_return_type(output: &ReturnType) -> Result<()> {
    let message = "`method` must return exactly `Result<*mut c_void, i32>`";

    let ReturnType::Type(_, ty) = output else {
        return Err(Error::new(output.span(), message));
    };

    let Type::Path(path) = ty.as_ref() else {
        return Err(Error::new_spanned(ty, message));
    };

    let Some(segment) = path.path.segments.last() else {
        return Err(Error::new_spanned(ty, message));
    };

    if segment.ident != "Result" {
        return Err(Error::new_spanned(ty, message));
    }

    let syn::PathArguments::AngleBracketed(generics) = &segment.arguments else {
        return Err(Error::new_spanned(ty, message));
    };

    let types: Vec<&Type> = generics
        .args
        .iter()
        .filter_map(|arg| match arg {
            syn::GenericArgument::Type(ty) => Some(ty),
            _ => None,
        })
        .collect();

    if types.len() != 2 {
        return Err(Error::new_spanned(ty, message));
    }

    // First generic must be `*mut c_void`.
    let Type::Ptr(pointer) = types[0] else {
        return Err(Error::new_spanned(ty, message));
    };
    if pointer.mutability.is_none() {
        return Err(Error::new_spanned(ty, message));
    }
    let Type::Path(inner) = pointer.elem.as_ref() else {
        return Err(Error::new_spanned(ty, message));
    };
    if !inner
        .path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "c_void")
    {
        return Err(Error::new_spanned(ty, message));
    }

    // Second generic must be `i32`.
    let Type::Path(inner) = types[1] else {
        return Err(Error::new_spanned(ty, message));
    };
    if !inner
        .path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "i32")
    {
        return Err(Error::new_spanned(ty, message));
    }

    Ok(())
}
