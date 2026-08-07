use quote::{format_ident, quote};
use syn::{
    Attribute, Error, Fields, Ident, ItemFn, ItemStruct, Meta, Path, Result, Type,
    parse::{Parse, ParseStream},
};

pub(crate) struct Args;

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            Ok(Self)
        } else {
            Err(input.error(
                "`component` no longer accepts arguments; declare fields in the plugin manifest",
            ))
        }
    }
}

pub(crate) struct CreatorArgs {
    component_type: Path,
}

impl Parse for CreatorArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let component_type = input.parse()?;
        if !input.is_empty() {
            return Err(input.error("`component_creator` only supports one component type"));
        }

        Ok(Self { component_type })
    }
}

enum FieldFunction {
    Generated,
    Custom,
}

struct Field {
    ident: Ident,
    ty: Type,
    getter: Option<FieldFunction>,
    serializer: Option<FieldFunction>,
    deserializer: Option<FieldFunction>,
}

pub(crate) fn expand(_args: Args, mut item: ItemStruct) -> Result<proc_macro2::TokenStream> {
    let component_ident = item.ident.clone();
    let component_id = component_ident.to_string();
    let component_fields = parse_component_fields(&mut item)?;

    let destroyer = create_destroyer_function(&component_ident, &component_id);
    let getters = create_getter_functions(&component_ident, &component_id, &component_fields);
    let serializers =
        create_serializer_functions(&component_ident, &component_id, &component_fields);
    let deserializers =
        create_deserializer_functions(&component_ident, &component_id, &component_fields);

    Ok(quote! {
        #item

        #destroyer
        #getters
        #serializers
        #deserializers
    })
}

pub(crate) fn expand_component_creator(
    args: CreatorArgs,
    item: ItemFn,
) -> Result<proc_macro2::TokenStream> {
    let component_type = args.component_type;
    let component_id = component_type
        .segments
        .last()
        .ok_or_else(|| Error::new_spanned(&component_type, "component type path cannot be empty"))?
        .ident
        .to_string();
    let creator_name = format!("wxr_create_{}", component_id);
    let creator_ident = format_ident!("{}", creator_name);
    let user_creator = item.sig.ident.clone();

    Ok(quote! {
        #item

        #[doc = "Generated WasserXR component creator binding."]
        #[doc = ""]
        #[doc = "Calls the annotated Rust creator and returns an owned component pointer or null."]
        #[doc = ""]
        #[doc = "# Safety"]
        #[doc = ""]
        #[doc = "`scene` must point to a valid `Scene`."]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn #creator_ident(
            scene: *mut ::wasserxr::scene::Scene,
        ) -> *mut ::std::ffi::c_void {
            if scene.is_null() {
                return ::std::ptr::null_mut();
            }

            match #user_creator(unsafe { &mut *scene }) {
                Some(component) => {
                    let component: #component_type = component;
                    Box::into_raw(Box::new(component)) as *mut ::std::ffi::c_void
                }
                None => ::std::ptr::null_mut(),
            }
        }
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

        let mut getter = None;
        let mut is_mutable = false;
        let mut serializer = None;
        let mut deserializer = None;
        let mut has_none = false;
        let mut kept_attrs = Vec::new();

        for attr in field.attrs.drain(..) {
            if attr.path().is_ident("getter") {
                getter = Some(parse_field_function_attribute(&attr, "getter")?);
            } else if attr.path().is_ident("mutable") {
                is_mutable = true;
            } else if attr.path().is_ident("serializer") {
                serializer = Some(parse_field_function_attribute(&attr, "serializer")?);
            } else if attr.path().is_ident("deserializer") {
                deserializer = Some(parse_field_function_attribute(&attr, "deserializer")?);
            } else if attr.path().is_ident("none") {
                has_none = true;
            } else {
                kept_attrs.push(attr);
            }
        }

        field.attrs = kept_attrs;

        let has_explicit_field_function =
            getter.is_some() || serializer.is_some() || deserializer.is_some();
        if has_none && (has_explicit_field_function || is_mutable) {
            return Err(Error::new_spanned(
                field,
                "`none` cannot be combined with field function attributes or `mutable`",
            ));
        }

        if !has_none && !has_explicit_field_function {
            getter = Some(FieldFunction::Generated);
            serializer = Some(FieldFunction::Generated);
            deserializer = Some(FieldFunction::Generated);
        }

        if is_mutable && getter.is_none() {
            getter = Some(FieldFunction::Generated);
        }

        component_fields.push(Field {
            ident: field_ident,
            ty: field.ty.clone(),
            getter,
            serializer,
            deserializer,
        });
    }

    Ok(component_fields)
}

fn parse_field_function_attribute(attr: &Attribute, name: &str) -> Result<FieldFunction> {
    match &attr.meta {
        Meta::Path(_) => Ok(FieldFunction::Generated),
        Meta::List(_) => {
            attr.parse_args::<Path>()?;
            Ok(FieldFunction::Custom)
        }
        Meta::NameValue(_) => Err(Error::new_spanned(
            attr,
            format!("`{name}` expects no value or a custom function path"),
        )),
    }
}

fn create_destroyer_function(
    component_ident: &Ident,
    component_id: &str,
) -> proc_macro2::TokenStream {
    let destroyer_name = format!("wxr_destroy_{}", component_id);
    let destroyer_ident = format_ident!("{}", destroyer_name);

    quote! {
        #[doc = "Generated WasserXR component destroyer binding."]
        #[doc = ""]
        #[doc = "Drops the component pointer previously returned by the generated component creator."]
        #[doc = ""]
        #[doc = "# Safety"]
        #[doc = ""]
        #[doc = "`ptr` must be a non-null pointer returned for this exact component type and must not be destroyed twice."]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn #destroyer_ident(ptr: *mut ::std::ffi::c_void) {
            unsafe {
                drop(Box::from_raw(ptr as *mut #component_ident));
            }
        }
    }
}

fn create_getter_functions(
    component_ident: &Ident,
    component_id: &str,
    fields: &[Field],
) -> proc_macro2::TokenStream {
    let getters = fields
        .iter()
        .filter(|field| matches!(&field.getter, Some(FieldFunction::Generated)))
        .map(|field| {
            let field_ident = &field.ident;
            let field_ty = &field.ty;
            let getter_name = format!("wxr_get_{}_{}", component_id, field_ident);
            let getter_ident = format_ident!("{}", getter_name);

            quote! {
                #[doc = "Generated WasserXR component field getter binding."]
                #[doc = ""]
                #[doc = "Returns a raw pointer to one field on the component value."]
                #[doc = ""]
                #[doc = "# Safety"]
                #[doc = ""]
                #[doc = "`ptr` must point to this exact component type. The returned pointer is only valid while the component is alive."]
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
        .filter(|field| matches!(&field.serializer, Some(FieldFunction::Generated)))
        .map(|field| {
            let field_ident = &field.ident;
            let field_ty = &field.ty;
            let serializer_name = format!("wxr_serialize_{}_{}", component_id, field_ident);
            let serializer_ident = format_ident!("{}", serializer_name);

            match component_field_serialization_kind(field_ty) {
                SerializationKind::Bytes => quote! {
                    #[doc = "Generated WasserXR component field serializer binding."]
                    #[doc = ""]
                    #[doc = "Serializes one numeric component field as little-endian bytes."]
                    #[doc = ""]
                    #[doc = "# Safety"]
                    #[doc = ""]
                    #[doc = "`ptr` must point to this exact component type."]
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
                },
                SerializationKind::Char => quote! {
                    #[doc = "Generated WasserXR component field serializer binding."]
                    #[doc = ""]
                    #[doc = "Serializes one `char` component field as little-endian `u32` bytes."]
                    #[doc = ""]
                    #[doc = "# Safety"]
                    #[doc = ""]
                    #[doc = "`ptr` must point to this exact component type."]
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
                },
                SerializationKind::String => quote! {
                    #[doc = "Generated WasserXR component field serializer binding."]
                    #[doc = ""]
                    #[doc = "Serializes one `String` component field as UTF-8 bytes."]
                    #[doc = ""]
                    #[doc = "# Safety"]
                    #[doc = ""]
                    #[doc = "`ptr` must point to this exact component type."]
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
                },
                SerializationKind::Bincode => quote! {
                    #[doc = "Generated WasserXR component field serializer binding."]
                    #[doc = ""]
                    #[doc = "Serializes one component field with serde and bincode."]
                    #[doc = ""]
                    #[doc = "# Safety"]
                    #[doc = ""]
                    #[doc = "`ptr` must point to this exact component type, and the field type must serialize correctly."]
                    #[allow(non_snake_case)]
                    pub unsafe extern "C" fn #serializer_ident(
                        ptr: *const ::std::ffi::c_void,
                    ) -> ::wasserxr::scene::component::SerializedBytes {
                        unsafe {
                            let value = &(*(ptr as *const #component_ident)).#field_ident;
                            ::wasserxr::scene::component::SerializedBytes::from_serializable(value)
                        }
                    }
                },
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
    let deserializers = fields
        .iter()
        .filter(|field| matches!(&field.deserializer, Some(FieldFunction::Generated)))
        .map(|field| {
            let field_ident = &field.ident;
            let field_ty = &field.ty;
            let deserializer_name = format!("wxr_deserialize_{}_{}", component_id, field_ident);
            let deserializer_ident = format_ident!("{}", deserializer_name);

            match component_field_serialization_kind(field_ty) {
                SerializationKind::Bytes => quote! {
                    #[doc = "Generated WasserXR component field deserializer binding."]
                    #[doc = ""]
                    #[doc = "Deserializes little-endian numeric bytes into one component field."]
                    #[doc = ""]
                    #[doc = "# Safety"]
                    #[doc = ""]
                    #[doc = "`ptr` must point to this exact component type, and `data` must be an owned `SerializedBytes` value."]
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
                },
                SerializationKind::Char => quote! {
                    #[doc = "Generated WasserXR component field deserializer binding."]
                    #[doc = ""]
                    #[doc = "Deserializes little-endian `u32` bytes into one `char` component field."]
                    #[doc = ""]
                    #[doc = "# Safety"]
                    #[doc = ""]
                    #[doc = "`ptr` must point to this exact component type, and `data` must be an owned `SerializedBytes` value."]
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
                },
                SerializationKind::String => quote! {
                    #[doc = "Generated WasserXR component field deserializer binding."]
                    #[doc = ""]
                    #[doc = "Deserializes UTF-8 bytes into one `String` component field."]
                    #[doc = ""]
                    #[doc = "# Safety"]
                    #[doc = ""]
                    #[doc = "`ptr` must point to this exact component type, and `data` must be an owned `SerializedBytes` value."]
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
                },
                SerializationKind::Bincode => quote! {
                    #[doc = "Generated WasserXR component field deserializer binding."]
                    #[doc = ""]
                    #[doc = "Deserializes one component field with serde and bincode."]
                    #[doc = ""]
                    #[doc = "# Safety"]
                    #[doc = ""]
                    #[doc = "`ptr` must point to this exact component type, and `data` must be an owned `SerializedBytes` value for that field type."]
                    #[allow(non_snake_case)]
                    pub unsafe extern "C" fn #deserializer_ident(
                        ptr: *mut ::std::ffi::c_void,
                        data: ::wasserxr::scene::component::SerializedBytes,
                    ) {
                        unsafe {
                            if let Some(value) = data.into_deserializable::<#field_ty>() {
                                (*(ptr as *mut #component_ident)).#field_ident = value;
                            }
                        }
                    }
                },
            }
        });

    quote! {
        #(#deserializers)*
    }
}

enum SerializationKind {
    Bytes,
    Char,
    String,
    Bincode,
}

fn component_field_serialization_kind(ty: &Type) -> SerializationKind {
    let Type::Path(path) = ty else {
        return SerializationKind::Bincode;
    };

    let Some(segment) = path.path.segments.last() else {
        return SerializationKind::Bincode;
    };

    match segment.ident.to_string().as_str() {
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
        | "usize" | "f32" | "f64" => SerializationKind::Bytes,
        "char" => SerializationKind::Char,
        "String" => SerializationKind::String,
        _ => SerializationKind::Bincode,
    }
}
