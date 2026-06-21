use quote::{format_ident, quote};
use syn::{Error, Fields, Ident, ItemStruct, Result, Type};

struct Field {
    ident: Ident,
    ty: Type,
    has_getter: bool,
    has_getter_mut: bool,
    has_setter: bool,
    has_mover: bool,
    has_taker: bool,
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
    let getters_mut =
        create_getter_mut_functions(&component_ident, &component_id, &component_fields);
    let setters = create_setter_functions(&component_ident, &component_id, &component_fields);
    let movers = create_mover_functions(&component_ident, &component_id, &component_fields);
    let takers = create_taker_functions(&component_ident, &component_id, &component_fields);
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
        #getters_mut
        #setters
        #movers
        #takers
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
        let mut has_getter_mut = false;
        let mut has_setter = false;
        let mut has_mover = false;
        let mut has_taker = false;
        let mut has_serializer = false;
        let mut has_deserializer = false;
        let mut has_none = false;
        let mut kept_attrs = Vec::new();

        for attr in field.attrs.drain(..) {
            if attr.path().is_ident("getter") {
                has_getter = true;
            } else if attr.path().is_ident("getter_mut") {
                has_getter_mut = true;
            } else if attr.path().is_ident("setter") {
                has_setter = true;
            } else if attr.path().is_ident("mover") {
                has_mover = true;
            } else if attr.path().is_ident("taker") {
                has_taker = true;
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

        let has_explicit_field_function = has_getter
            || has_getter_mut
            || has_setter
            || has_mover
            || has_taker
            || has_serializer
            || has_deserializer;
        if has_none && has_explicit_field_function {
            return Err(Error::new_spanned(
                field,
                "`none` cannot be combined with field function attributes",
            ));
        }

        if !has_none && !has_explicit_field_function {
            has_getter = true;
            has_getter_mut = true;
            has_setter = true;
            has_serializer = true;
            has_deserializer = true;
        }

        component_fields.push(Field {
            ident: field_ident,
            ty: field.ty.clone(),
            has_getter,
            has_getter_mut,
            has_setter,
            has_mover,
            has_taker,
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
        let getter_mut = if field.has_getter_mut {
            let getter_mut_ident = format_ident!("wxr_get_mut_{}_{}", component_id, field_ident);
            quote! { Some(#getter_mut_ident) }
        } else {
            quote! { None }
        };
        let setter = if field.has_setter {
            let setter_ident = format_ident!("wxr_set_{}_{}", component_id, field_ident);
            quote! { Some(#setter_ident) }
        } else {
            quote! { None }
        };
        let mover = if field.has_mover {
            let mover_ident = format_ident!("wxr_move_{}_{}", component_id, field_ident);
            quote! { Some(#mover_ident) }
        } else {
            quote! { None }
        };
        let taker = if field.has_taker {
            let taker_ident = format_ident!("wxr_take_{}_{}", component_id, field_ident);
            quote! { Some(#taker_ident) }
        } else {
            quote! { None }
        };
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
                #getter_mut,
                #setter,
                #mover,
                #taker,
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

fn create_getter_mut_functions(
    component_ident: &Ident,
    component_id: &str,
    fields: &[Field],
) -> proc_macro2::TokenStream {
    let getters_mut = fields
        .iter()
        .filter(|field| field.has_getter_mut)
        .map(|field| {
            let field_ident = &field.ident;
            let field_ty = &field.ty;
            let getter_mut_name = format!("wxr_get_mut_{}_{}", component_id, field_ident);
            let getter_mut_ident = format_ident!("{}", getter_mut_name);

            quote! {
                #[unsafe(export_name = #getter_mut_name)]
                #[allow(non_snake_case)]
                pub unsafe extern "C" fn #getter_mut_ident(
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
        #(#getters_mut)*
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

fn create_mover_functions(
    component_ident: &Ident,
    component_id: &str,
    fields: &[Field],
) -> proc_macro2::TokenStream {
    let movers = fields.iter().filter(|field| field.has_mover).map(|field| {
        let field_ident = &field.ident;
        let field_ty = &field.ty;
        let mover_name = format!("wxr_move_{}_{}", component_id, field_ident);
        let mover_ident = format_ident!("{}", mover_name);

        quote! {
            #[unsafe(export_name = #mover_name)]
            #[allow(non_snake_case)]
            pub unsafe extern "C" fn #mover_ident(
                ptr: *mut ::std::ffi::c_void,
                data: *mut ::std::ffi::c_void,
            ) {
                unsafe {
                    (*(ptr as *mut #component_ident)).#field_ident =
                        *Box::from_raw(data as *mut #field_ty);
                }
            }
        }
    });

    quote! {
        #(#movers)*
    }
}

fn create_taker_functions(
    component_ident: &Ident,
    component_id: &str,
    fields: &[Field],
) -> proc_macro2::TokenStream {
    let takers = fields.iter().filter(|field| field.has_taker).map(|field| {
        let field_ident = &field.ident;
        let field_ty = &field.ty;
        let taker_name = format!("wxr_take_{}_{}", component_id, field_ident);
        let taker_ident = format_ident!("{}", taker_name);

        quote! {
            #[unsafe(export_name = #taker_name)]
            #[allow(non_snake_case)]
            pub unsafe extern "C" fn #taker_ident(
                ptr: *mut ::std::ffi::c_void,
                out: *mut ::std::ffi::c_void,
            ) {
                unsafe {
                    ::std::ptr::write(
                        out as *mut #field_ty,
                        ::std::mem::take(
                            &mut (*(ptr as *mut #component_ident)).#field_ident,
                        ),
                    );
                }
            }
        }
    });

    quote! {
        #(#takers)*
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
