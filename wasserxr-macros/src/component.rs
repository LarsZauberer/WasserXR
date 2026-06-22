use quote::{format_ident, quote};
use syn::{Error, Fields, Ident, ItemStruct, Result, Type};

struct Field {
    ident: Ident,
    ty: Type,
    has_getter: bool,
    is_mutable: bool,
    has_serializer: bool,
    has_deserializer: bool,
}

pub(crate) fn expand(mut item: ItemStruct) -> Result<proc_macro2::TokenStream> {
    let component_ident = item.ident.clone();
    let component_id = component_ident.to_string();
    let component_fields = parse_component_fields(&mut item)?;

    let creator = create_creator_function(&component_ident, &component_id);
    let destroyer = create_destroyer_function(&component_ident, &component_id);
    let schema = create_schema_function(&component_id, &component_fields);
    let getters = create_getter_functions(&component_ident, &component_id, &component_fields);
    let serializers =
        create_serializer_functions(&component_ident, &component_id, &component_fields);
    let deserializers =
        create_deserializer_functions(&component_ident, &component_id, &component_fields);

    Ok(quote! {
        #item

        #creator
        #destroyer
        #schema
        #getters
        #serializers
        #deserializers
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
        let mut is_mutable = false;
        let mut has_serializer = false;
        let mut has_deserializer = false;
        let mut has_none = false;
        let mut kept_attrs = Vec::new();

        for attr in field.attrs.drain(..) {
            if attr.path().is_ident("getter") {
                has_getter = true;
            } else if attr.path().is_ident("mutable") {
                is_mutable = true;
            } else if attr.path().is_ident("serializer") {
                has_serializer = true;
            } else if attr.path().is_ident("deserializer") {
                has_deserializer = true;
            } else if attr.path().is_ident("none") {
                has_none = true;
            } else {
                kept_attrs.push(attr);
            }
        }

        field.attrs = kept_attrs;

        let has_explicit_field_function = has_getter || has_serializer || has_deserializer;
        if has_none && (has_explicit_field_function || is_mutable) {
            return Err(Error::new_spanned(
                field,
                "`none` cannot be combined with field function attributes or `mutable`",
            ));
        }

        if !has_none && !has_explicit_field_function {
            has_getter = true;
            has_serializer = true;
            has_deserializer = true;
        }

        if is_mutable {
            has_getter = true;
        }

        component_fields.push(Field {
            ident: field_ident,
            ty: field.ty.clone(),
            has_getter,
            is_mutable,
            has_serializer,
            has_deserializer,
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
        let mutable = field.is_mutable;
        let serializer = if field.has_serializer
            && component_field_serialization_kind(&field.ty).is_some()
        {
            let serializer_ident = format_ident!("wxr_serialize_{}_{}", component_id, field_ident);
            quote! { Some(#serializer_ident) }
        } else {
            quote! { None }
        };
        let deserializer =
            if field.has_deserializer && component_field_serialization_kind(&field.ty).is_some() {
                let deserializer_ident =
                    format_ident!("wxr_deserialize_{}_{}", component_id, field_ident);
                quote! { Some(#deserializer_ident) }
            } else {
                quote! { None }
            };

        quote! {
            (*schema).add_field(
                #field_name.to_owned(),
                #field_type,
                #getter,
                #mutable,
                #serializer,
                #deserializer,
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
                ptr: *mut ::std::ffi::c_void,
            ) -> *mut ::std::ffi::c_void {
                unsafe {
                    &mut (*(ptr as *mut #component_ident)).#field_ident
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

fn create_serializer_functions(
    component_ident: &Ident,
    component_id: &str,
    fields: &[Field],
) -> proc_macro2::TokenStream {
    let serializers = fields
        .iter()
        .filter(|field| field.has_serializer)
        .filter_map(|field| {
            let field_ident = &field.ident;
            let field_ty = &field.ty;
            let serializer_name = format!("wxr_serialize_{}_{}", component_id, field_ident);
            let serializer_ident = format_ident!("{}", serializer_name);

            match component_field_serialization_kind(field_ty)? {
                SerializationKind::Bytes => Some(quote! {
                    #[unsafe(export_name = #serializer_name)]
                    #[allow(non_snake_case)]
                    pub unsafe extern "C" fn #serializer_ident(
                        ptr: *const ::std::ffi::c_void,
                    ) -> ::wasserxr::scene::component::SerializedBytes {
                        unsafe {
                            let value = &(*(ptr as *const #component_ident)).#field_ident;
                            ::wasserxr::scene::component::SerializedBytes::from_vec(
                                value.to_le_bytes().to_vec(),
                            )
                        }
                    }
                }),
                SerializationKind::Char => Some(quote! {
                    #[unsafe(export_name = #serializer_name)]
                    #[allow(non_snake_case)]
                    pub unsafe extern "C" fn #serializer_ident(
                        ptr: *const ::std::ffi::c_void,
                    ) -> ::wasserxr::scene::component::SerializedBytes {
                        unsafe {
                            let value = (*(ptr as *const #component_ident)).#field_ident as u32;
                            ::wasserxr::scene::component::SerializedBytes::from_vec(
                                value.to_le_bytes().to_vec(),
                            )
                        }
                    }
                }),
                SerializationKind::String => Some(quote! {
                    #[unsafe(export_name = #serializer_name)]
                    #[allow(non_snake_case)]
                    pub unsafe extern "C" fn #serializer_ident(
                        ptr: *const ::std::ffi::c_void,
                    ) -> ::wasserxr::scene::component::SerializedBytes {
                        unsafe {
                            let value = &(*(ptr as *const #component_ident)).#field_ident;
                            ::wasserxr::scene::component::SerializedBytes::from_vec(
                                value.as_bytes().to_vec(),
                            )
                        }
                    }
                }),
            }
        });

    quote! {
        #(#serializers)*
    }
}

fn create_deserializer_functions(
    component_ident: &Ident,
    component_id: &str,
    fields: &[Field],
) -> proc_macro2::TokenStream {
    let deserializers = fields.iter().filter(|field| field.has_deserializer).filter_map(|field| {
        let field_ident = &field.ident;
        let field_ty = &field.ty;
        let deserializer_name = format!("wxr_deserialize_{}_{}", component_id, field_ident);
        let deserializer_ident = format_ident!("{}", deserializer_name);

        match component_field_serialization_kind(field_ty)? {
            SerializationKind::Bytes => Some(quote! {
                #[unsafe(export_name = #deserializer_name)]
                #[allow(non_snake_case)]
                pub unsafe extern "C" fn #deserializer_ident(
                    ptr: *mut ::std::ffi::c_void,
                    data: ::wasserxr::scene::component::SerializedBytes,
                ) {
                    unsafe {
                        let bytes = data.into_vec();
                        if let Ok(bytes) =
                            <[u8; ::std::mem::size_of::<#field_ty>()]>::try_from(bytes.as_slice())
                        {
                            (*(ptr as *mut #component_ident)).#field_ident =
                                #field_ty::from_le_bytes(bytes);
                        }
                    }
                }
            }),
            SerializationKind::Char => Some(quote! {
                #[unsafe(export_name = #deserializer_name)]
                #[allow(non_snake_case)]
                pub unsafe extern "C" fn #deserializer_ident(
                    ptr: *mut ::std::ffi::c_void,
                    data: ::wasserxr::scene::component::SerializedBytes,
                ) {
                    unsafe {
                        let bytes = data.into_vec();
                        if let Ok(bytes) = <[u8; 4]>::try_from(bytes.as_slice()) {
                            if let Some(value) = ::std::char::from_u32(u32::from_le_bytes(bytes)) {
                                (*(ptr as *mut #component_ident)).#field_ident = value;
                            }
                        }
                    }
                }
            }),
            SerializationKind::String => Some(quote! {
                #[unsafe(export_name = #deserializer_name)]
                #[allow(non_snake_case)]
                pub unsafe extern "C" fn #deserializer_ident(
                    ptr: *mut ::std::ffi::c_void,
                    data: ::wasserxr::scene::component::SerializedBytes,
                ) {
                    unsafe {
                        let bytes = data.into_vec();
                        if let Ok(value) = ::std::string::String::from_utf8(bytes) {
                            (*(ptr as *mut #component_ident)).#field_ident = value;
                        }
                    }
                }
            }),
        }
    });

    quote! {
        #(#deserializers)*
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

enum SerializationKind {
    Bytes,
    Char,
    String,
}

fn component_field_serialization_kind(ty: &Type) -> Option<SerializationKind> {
    let Type::Path(path) = ty else {
        return None;
    };

    let segment = path.path.segments.last()?;

    match segment.ident.to_string().as_str() {
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
        | "usize" | "f32" | "f64" => Some(SerializationKind::Bytes),
        "char" => Some(SerializationKind::Char),
        "String" => Some(SerializationKind::String),
        _ => None,
    }
}
