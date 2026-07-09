use quote::{format_ident, quote};
use syn::{
    Attribute, Error, Fields, Ident, ItemFn, ItemStruct, Path, Result, Type,
    parse::{Parse, ParseStream},
};

pub(crate) struct CreatorArgs {
    asset_type: Path,
}

impl Parse for CreatorArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let asset_type = input.parse()?;
        if !input.is_empty() {
            return Err(input.error("`asset_type_creator` only supports one asset type"));
        }

        Ok(Self { asset_type })
    }
}

struct Field {
    ident: Ident,
    ty: Type,
}

pub(crate) fn expand_asset_type(mut item: ItemStruct) -> Result<proc_macro2::TokenStream> {
    let asset_ident = item.ident.clone();
    let asset_id = asset_ident.to_string();
    let fields = parse_asset_fields(&mut item)?;

    let destroyer = create_destroyer_function(&asset_ident, &asset_id);
    let schema = create_schema_function(&asset_id, &fields);
    let getters = create_getter_functions(&asset_ident, &asset_id, &fields);

    Ok(quote! {
        #item

        #destroyer
        #schema
        #getters
    })
}

pub(crate) fn expand_asset_type_creator(
    args: CreatorArgs,
    item: ItemFn,
) -> Result<proc_macro2::TokenStream> {
    let asset_type = args.asset_type;
    let asset_id = asset_type
        .segments
        .last()
        .ok_or_else(|| Error::new_spanned(&asset_type, "asset type path cannot be empty"))?
        .ident
        .to_string();
    let creator_name = format!("wxr_asset_create_{}", asset_id);
    let creator_ident = format_ident!("{}", creator_name);
    let user_creator = item.sig.ident.clone();

    Ok(quote! {
        #item

        #[doc = "Generated WasserXR asset creator binding."]
        #[doc = ""]
        #[doc = "Converts the C string data argument to `&str`, calls the annotated Rust creator, and returns an owned asset pointer or null."]
        #[doc = ""]
        #[doc = "# Safety"]
        #[doc = ""]
        #[doc = "`scene` must point to a valid `Scene`, and `data` must point to a valid nul-terminated UTF-8 C string."]
        #[unsafe(export_name = #creator_name)]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn #creator_ident(
            scene: *mut ::wasserxr::scene::Scene,
            data: *const ::std::ffi::c_char,
        ) -> *mut ::std::ffi::c_void {
            if scene.is_null() {
                return ::std::ptr::null_mut();
            }

            if data.is_null() {
                return ::std::ptr::null_mut();
            }

            let Ok(data) = unsafe { ::std::ffi::CStr::from_ptr(data) }.to_str() else {
                return ::std::ptr::null_mut();
            };

            match #user_creator(unsafe { &mut *scene }, data) {
                Some(asset) => {
                    let asset: #asset_type = asset;
                    Box::into_raw(Box::new(asset)) as *mut ::std::ffi::c_void
                }
                None => ::std::ptr::null_mut(),
            }
        }
    })
}

fn parse_asset_fields(item: &mut ItemStruct) -> Result<Vec<Field>> {
    let Fields::Named(fields) = &mut item.fields else {
        return Err(Error::new_spanned(
            item,
            "`asset_type` only supports structs with named fields",
        ));
    };

    let mut asset_fields = Vec::new();

    for field in fields.named.iter_mut() {
        let Some(field_ident) = field.ident.clone() else {
            return Err(Error::new_spanned(
                field,
                "`asset_type` only supports named fields",
            ));
        };

        let mut has_none = false;
        let mut kept_attrs = Vec::new();

        for attr in field.attrs.drain(..) {
            if attr.path().is_ident("none") {
                has_none = true;
            } else {
                reject_component_only_asset_attribute(&attr)?;
                kept_attrs.push(attr);
            }
        }

        field.attrs = kept_attrs;

        if !has_none {
            asset_fields.push(Field {
                ident: field_ident,
                ty: field.ty.clone(),
            });
        }
    }

    Ok(asset_fields)
}

fn reject_component_only_asset_attribute(attr: &Attribute) -> Result<()> {
    let attr_name = attr
        .path()
        .get_ident()
        .map(Ident::to_string)
        .unwrap_or_default();

    match attr_name.as_str() {
        "getter" | "mutable" | "serializer" | "deserializer" => Err(Error::new_spanned(
            attr,
            "`asset_type` only supports `#[none]` field attributes",
        )),
        _ => {
            let _ = &attr.meta;
            Ok(())
        }
    }
}

fn create_destroyer_function(asset_ident: &Ident, asset_id: &str) -> proc_macro2::TokenStream {
    let destroyer_name = format!("wxr_asset_destroy_{}", asset_id);
    let destroyer_ident = format_ident!("{}", destroyer_name);

    quote! {
        #[doc = "Generated WasserXR asset destroyer binding."]
        #[doc = ""]
        #[doc = "Drops the asset pointer previously returned by the generated asset creator."]
        #[doc = ""]
        #[doc = "# Safety"]
        #[doc = ""]
        #[doc = "`ptr` must be a non-null pointer returned for this exact asset type and must not be destroyed twice."]
        #[unsafe(export_name = #destroyer_name)]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn #destroyer_ident(
            _scene: *mut ::wasserxr::scene::Scene,
            ptr: *mut ::std::ffi::c_void,
        ) {
            unsafe {
                drop(Box::from_raw(ptr as *mut #asset_ident));
            }
        }
    }
}

fn create_schema_function(asset_id: &str, fields: &[Field]) -> proc_macro2::TokenStream {
    let schema_name = format!("wxr_asset_schema_{}", asset_id);
    let schema_ident = format_ident!("{}", schema_name);
    let schema_fields = fields.iter().map(|field| {
        let field_ident = &field.ident;
        let field_name = field_ident.to_string();
        let field_type = asset_field_type(&field.ty);
        let getter_ident = format_ident!("wxr_asset_get_{}_{}", asset_id, field_ident);

        quote! {
            (*schema).add_field(
                #field_name.to_owned(),
                #field_type,
                Some(#getter_ident),
            );
        }
    });

    quote! {
        #[doc = "Generated WasserXR asset schema binding."]
        #[doc = ""]
        #[doc = "Registers every queryable asset field and its getter in the provided schema."]
        #[doc = ""]
        #[doc = "# Safety"]
        #[doc = ""]
        #[doc = "`schema` must point to a valid mutable asset schema."]
        #[unsafe(export_name = #schema_name)]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn #schema_ident(
            schema: *mut ::wasserxr::scene::assets::Schema,
        ) {
            unsafe {
                #(#schema_fields)*
            }
        }
    }
}

fn create_getter_functions(
    asset_ident: &Ident,
    asset_id: &str,
    fields: &[Field],
) -> proc_macro2::TokenStream {
    let getters = fields.iter().map(|field| {
        let field_ident = &field.ident;
        let field_ty = &field.ty;
        let getter_name = format!("wxr_asset_get_{}_{}", asset_id, field_ident);
        let getter_ident = format_ident!("{}", getter_name);

        quote! {
            #[doc = "Generated WasserXR asset field getter binding."]
            #[doc = ""]
            #[doc = "Returns a raw pointer to one field on the asset value."]
            #[doc = ""]
            #[doc = "# Safety"]
            #[doc = ""]
            #[doc = "`ptr` must point to this exact asset type. The returned pointer is only valid while the asset is alive."]
            #[unsafe(export_name = #getter_name)]
            #[allow(non_snake_case)]
            pub unsafe extern "C" fn #getter_ident(
                ptr: *mut ::std::ffi::c_void,
            ) -> *mut ::std::ffi::c_void {
                unsafe {
                    &mut (*(ptr as *mut #asset_ident)).#field_ident
                        as *mut #field_ty
                        as *mut ::std::ffi::c_void
                }
            }
        }
    });

    quote! {
        #(#getters)*
    }
}

fn asset_field_type(ty: &Type) -> proc_macro2::TokenStream {
    if let Some(field_type) = array_field_type(ty) {
        return field_type;
    }

    let Type::Path(path) = ty else {
        return quote! { ::wasserxr::scene::component::FieldType::Blob };
    };

    let Some(segment) = path.path.segments.last() else {
        return quote! { ::wasserxr::scene::component::FieldType::Blob };
    };

    match segment.ident.to_string().as_str() {
        "i8" => quote! { ::wasserxr::scene::component::FieldType::I8 },
        "i16" => quote! { ::wasserxr::scene::component::FieldType::I16 },
        "i32" => quote! { ::wasserxr::scene::component::FieldType::I32 },
        "i64" => quote! { ::wasserxr::scene::component::FieldType::I64 },
        "i128" => quote! { ::wasserxr::scene::component::FieldType::I128 },
        "isize" => quote! { ::wasserxr::scene::component::FieldType::Isize },
        "u8" => quote! { ::wasserxr::scene::component::FieldType::U8 },
        "u16" => quote! { ::wasserxr::scene::component::FieldType::U16 },
        "u32" => quote! { ::wasserxr::scene::component::FieldType::U32 },
        "u64" => quote! { ::wasserxr::scene::component::FieldType::U64 },
        "u128" => quote! { ::wasserxr::scene::component::FieldType::U128 },
        "usize" => quote! { ::wasserxr::scene::component::FieldType::Usize },
        "f32" => quote! { ::wasserxr::scene::component::FieldType::F32 },
        "f64" => quote! { ::wasserxr::scene::component::FieldType::F64 },
        "char" => quote! { ::wasserxr::scene::component::FieldType::Char },
        "String" => quote! { ::wasserxr::scene::component::FieldType::String },
        "bool" => quote! { ::wasserxr::scene::component::FieldType::Boolean },
        _ => quote! { ::wasserxr::scene::component::FieldType::Blob },
    }
}

fn array_field_type(ty: &Type) -> Option<proc_macro2::TokenStream> {
    let Type::Array(array) = ty else {
        return None;
    };
    let Type::Path(element) = array.elem.as_ref() else {
        return None;
    };
    let syn::Expr::Lit(length) = &array.len else {
        return None;
    };
    let syn::Lit::Int(length) = &length.lit else {
        return None;
    };

    match (
        element.path.segments.last()?.ident.to_string().as_str(),
        length.base10_parse::<usize>().ok()?,
    ) {
        ("f32", 2) => Some(quote! { ::wasserxr::scene::component::FieldType::F32Vec2 }),
        ("f32", 3) => Some(quote! { ::wasserxr::scene::component::FieldType::F32Vec3 }),
        ("f64", 2) => Some(quote! { ::wasserxr::scene::component::FieldType::F64Vec2 }),
        ("f64", 3) => Some(quote! { ::wasserxr::scene::component::FieldType::F64Vec3 }),
        _ => None,
    }
}
