use quote::{format_ident, quote};
use syn::{Error, Fields, Ident, ItemStruct, Result, Type};

struct Field {
    ident: Ident,
    ty: Type,
    has_getter: bool,
    has_setter: bool,
}

pub(crate) fn expand(mut item: ItemStruct) -> Result<proc_macro2::TokenStream> {
    let component_ident = item.ident.clone();
    let component_id = component_ident.to_string();
    let component_fields = parse_component_fields(&mut item)?;

    let creator = create_creator_function(&component_ident, &component_id);
    let destroyer = create_destroyer_function(&component_ident, &component_id);
    let schema = create_schema_function(&component_id, &component_fields);
    let getters = create_getter_functions(&component_ident, &component_id, &component_fields);
    let setters = create_setter_functions(&component_ident, &component_id, &component_fields);

    Ok(quote! {
        #item

        #creator
        #destroyer
        #schema
        #getters
        #setters
    })
}

fn parse_component_fields(item: &mut ItemStruct) -> Result<Vec<Field>> {
    let Fields::Named(fields) = &mut item.fields else {
        return Err(Error::new_spanned(
            item,
            "`component` only supports structs with named fields",
        ));
    };

    let mut component_fields = Vec::new();

    for field in fields.named.iter_mut() {
        let Some(field_ident) = field.ident.clone() else {
            return Err(Error::new_spanned(
                field,
                "`component` only supports named fields",
            ));
        };

        let mut has_getter = false;
        let mut has_setter = false;
        let mut kept_attrs = Vec::new();

        for attr in field.attrs.drain(..) {
            if attr.path().is_ident("getter") {
                has_getter = true;
            } else if attr.path().is_ident("setter") {
                has_setter = true;
            } else {
                kept_attrs.push(attr);
            }
        }

        field.attrs = kept_attrs;

        component_fields.push(Field {
            ident: field_ident,
            ty: field.ty.clone(),
            has_getter,
            has_setter,
        });
    }

    Ok(component_fields)
}

fn create_creator_function(
    component_ident: &Ident,
    component_id: &str,
) -> proc_macro2::TokenStream {
    let creator_name = format!("wxr_create_{}", component_id);
    let creator_ident = format_ident!("{}", creator_name);

    quote! {
        #[unsafe(export_name = #creator_name)]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn #creator_ident() -> *mut ::std::ffi::c_void {
            Box::into_raw(Box::new(#component_ident::default())) as *mut ::std::ffi::c_void
        }
    }
}

fn create_destroyer_function(
    component_ident: &Ident,
    component_id: &str,
) -> proc_macro2::TokenStream {
    let destroyer_name = format!("wxr_destroy_{}", component_id);
    let destroyer_ident = format_ident!("{}", destroyer_name);

    quote! {
        #[unsafe(export_name = #destroyer_name)]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn #destroyer_ident(ptr: *mut ::std::ffi::c_void) {
            unsafe {
                drop(Box::from_raw(ptr as *mut #component_ident));
            }
        }
    }
}

fn create_schema_function(component_id: &str, fields: &[Field]) -> proc_macro2::TokenStream {
    let schema_name = format!("wxr_schema_{}", component_id);
    let schema_ident = format_ident!("{}", schema_name);
    let schema_fields = fields.iter().map(|field| {
        let field_ident = &field.ident;
        let field_name = field_ident.to_string();
        let field_type = component_field_type(&field.ty);
        let getter = if field.has_getter {
            let getter_ident = format_ident!("wxr_get_{}_{}", component_id, field_ident);
            quote! { Some(#getter_ident) }
        } else {
            quote! { None }
        };
        let setter = if field.has_setter {
            let setter_ident = format_ident!("wxr_set_{}_{}", component_id, field_ident);
            quote! { Some(#setter_ident) }
        } else {
            quote! { None }
        };

        quote! {
            (*schema).add_field(
                #field_name.to_owned(),
                #field_type,
                #getter,
                #setter,
            );
        }
    });

    quote! {
        #[unsafe(export_name = #schema_name)]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn #schema_ident(
            schema: *mut ::wasserxr::scene::component::Schema,
        ) {
            unsafe {
                #(#schema_fields)*
            }
        }
    }
}

fn create_getter_functions(
    component_ident: &Ident,
    component_id: &str,
    fields: &[Field],
) -> proc_macro2::TokenStream {
    let getters = fields.iter().filter(|field| field.has_getter).map(|field| {
        let field_ident = &field.ident;
        let field_ty = &field.ty;
        let getter_name = format!("wxr_get_{}_{}", component_id, field_ident);
        let getter_ident = format_ident!("{}", getter_name);

        quote! {
            #[unsafe(export_name = #getter_name)]
            #[allow(non_snake_case)]
            pub unsafe extern "C" fn #getter_ident(
                ptr: *const ::std::ffi::c_void,
            ) -> *const ::std::ffi::c_void {
                unsafe {
                    &(*(ptr as *const #component_ident)).#field_ident
                        as *const #field_ty
                        as *const ::std::ffi::c_void
                }
            }
        }
    });

    quote! {
        #(#getters)*
    }
}

fn create_setter_functions(
    component_ident: &Ident,
    component_id: &str,
    fields: &[Field],
) -> proc_macro2::TokenStream {
    let setters = fields.iter().filter(|field| field.has_setter).map(|field| {
        let field_ident = &field.ident;
        let field_ty = &field.ty;
        let setter_name = format!("wxr_set_{}_{}", component_id, field_ident);
        let setter_ident = format_ident!("{}", setter_name);

        quote! {
            #[unsafe(export_name = #setter_name)]
            #[allow(non_snake_case)]
            pub unsafe extern "C" fn #setter_ident(
                ptr: *mut ::std::ffi::c_void,
                data: *const ::std::ffi::c_void,
            ) {
                unsafe {
                    (*(ptr as *mut #component_ident)).#field_ident =
                        (*(data as *const #field_ty)).clone();
                }
            }
        }
    });

    quote! {
        #(#setters)*
    }
}

fn component_field_type(ty: &Type) -> proc_macro2::TokenStream {
    let Type::Path(path) = ty else {
        return quote! { ::wasserxr::scene::component::FieldType::Blob };
    };

    let Some(segment) = path.path.segments.last() else {
        return quote! { ::wasserxr::scene::component::FieldType::Blob };
    };

    match segment.ident.to_string().as_str() {
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
        | "usize" => quote! { ::wasserxr::scene::component::FieldType::Long },
        "f32" | "f64" => quote! { ::wasserxr::scene::component::FieldType::Float },
        "char" => quote! { ::wasserxr::scene::component::FieldType::Char },
        "String" => quote! { ::wasserxr::scene::component::FieldType::String },
        _ => quote! { ::wasserxr::scene::component::FieldType::Blob },
    }
}
