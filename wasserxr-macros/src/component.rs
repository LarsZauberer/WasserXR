use quote::{format_ident, quote};
use syn::{
    Attribute, Error, Fields, Ident, ItemFn, ItemStruct, Meta, Path, Result, Token, Type,
    parse::{Parse, ParseStream},
};

#[derive(Default)]
pub(crate) struct Args {
    no_schema: bool,
}

impl Parse for Args {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.is_empty() {
            return Ok(Args::default());
        }

        let ident: Ident = input.parse()?;
        if ident != "no_schema" {
            return Err(Error::new_spanned(
                ident,
                "`component` only supports the `no_schema` argument",
            ));
        }

        if !input.is_empty() {
            return Err(input.error("`component` only supports the `no_schema` argument"));
        }

        Ok(Args { no_schema: true })
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
    Custom(Path),
}

struct Field {
    ident: Ident,
    ty: Type,
    getter: Option<FieldFunction>,
    is_mutable: bool,
    serializer: Option<FieldFunction>,
    deserializer: Option<FieldFunction>,
}

struct VirtualField {
    ident: Ident,
    ty: Type,
    getter: Path,
    is_mutable: bool,
}

impl Parse for VirtualField {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let ty = input.parse()?;
        let mut getter = None;
        let mut is_mutable = false;

        while !input.is_empty() {
            input.parse::<Token![,]>()?;
            if input.is_empty() {
                break;
            }

            let key: Ident = input.parse()?;
            if key == "getter" {
                if getter.is_some() {
                    return Err(Error::new_spanned(
                        key,
                        "virtual field getter is duplicated",
                    ));
                }
                input.parse::<Token![=]>()?;
                getter = Some(input.parse()?);
            } else if key == "mutable" {
                if is_mutable {
                    return Err(Error::new_spanned(
                        key,
                        "virtual field mutable marker is duplicated",
                    ));
                }
                is_mutable = true;
            } else {
                return Err(Error::new_spanned(
                    key,
                    "virtual field only supports `getter = ...` and `mutable`",
                ));
            }
        }

        let Some(getter) = getter else {
            return Err(input.error("virtual field requires `getter = ...`"));
        };

        Ok(VirtualField {
            ident,
            ty,
            getter,
            is_mutable,
        })
    }
}

pub(crate) fn expand(args: Args, mut item: ItemStruct) -> Result<proc_macro2::TokenStream> {
    let component_ident = item.ident.clone();
    let component_id = component_ident.to_string();
    let virtual_fields = parse_virtual_fields(&mut item)?;
    if args.no_schema && !virtual_fields.is_empty() {
        return Err(Error::new_spanned(
            item.ident,
            "`virtual_field` cannot be used with `component(no_schema)`",
        ));
    }
    let component_fields = parse_component_fields(&mut item)?;

    let destroyer = create_destroyer_function(&component_ident, &component_id);
    let schema = if args.no_schema {
        quote! {}
    } else {
        create_schema_function(&component_id, &component_fields, &virtual_fields)
    };
    let getters = create_getter_functions(&component_ident, &component_id, &component_fields);
    let serializers =
        create_serializer_functions(&component_ident, &component_id, &component_fields);
    let deserializers =
        create_deserializer_functions(&component_ident, &component_id, &component_fields);

    Ok(quote! {
        #item

        #destroyer
        #schema
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

        #[unsafe(export_name = #creator_name)]
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

fn parse_virtual_fields(item: &mut ItemStruct) -> Result<Vec<VirtualField>> {
    let mut virtual_fields = Vec::new();
    let mut kept_attrs = Vec::new();

    for attr in item.attrs.drain(..) {
        if attr.path().is_ident("virtual_field") {
            virtual_fields.push(attr.parse_args()?);
        } else {
            kept_attrs.push(attr);
        }
    }

    item.attrs = kept_attrs;
    Ok(virtual_fields)
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
            is_mutable,
            serializer,
            deserializer,
        });
    }

    Ok(component_fields)
}

fn parse_field_function_attribute(attr: &Attribute, name: &str) -> Result<FieldFunction> {
    match &attr.meta {
        Meta::Path(_) => Ok(FieldFunction::Generated),
        Meta::List(_) => Ok(FieldFunction::Custom(attr.parse_args()?)),
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
        #[unsafe(export_name = #destroyer_name)]
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn #destroyer_ident(ptr: *mut ::std::ffi::c_void) {
            unsafe {
                drop(Box::from_raw(ptr as *mut #component_ident));
            }
        }
    }
}

fn create_schema_function(
    component_id: &str,
    fields: &[Field],
    virtual_fields: &[VirtualField],
) -> proc_macro2::TokenStream {
    let schema_name = format!("wxr_schema_{}", component_id);
    let schema_ident = format_ident!("{}", schema_name);
    let schema_fields = fields.iter().map(|field| {
        let field_ident = &field.ident;
        let field_name = field_ident.to_string();
        let field_type = component_field_type(&field.ty);
        let getter = create_getter_option(component_id, field);
        let mutable = field.is_mutable;
        let serializer = create_serializer_option(component_id, field);
        let deserializer = create_deserializer_option(component_id, field);

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
    let schema_virtual_fields = virtual_fields.iter().map(|field| {
        let field_name = field.ident.to_string();
        let field_type = component_field_type(&field.ty);
        let getter = &field.getter;
        let mutable = field.is_mutable;

        quote! {
            (*schema).add_field(
                #field_name.to_owned(),
                #field_type,
                {
                    let getter: ::wasserxr::scene::component::Getter = #getter;
                    Some(getter)
                },
                #mutable,
                None,
                None,
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
                #(#schema_virtual_fields)*
            }
        }
    }
}

fn create_getter_option(component_id: &str, field: &Field) -> proc_macro2::TokenStream {
    match &field.getter {
        Some(FieldFunction::Generated) => {
            let field_ident = &field.ident;
            let getter_ident = format_ident!("wxr_get_{}_{}", component_id, field_ident);
            quote! { Some(#getter_ident) }
        }
        Some(FieldFunction::Custom(getter)) => quote! {{
            let getter: ::wasserxr::scene::component::Getter = #getter;
            Some(getter)
        }},
        None => quote! { None },
    }
}

fn create_serializer_option(component_id: &str, field: &Field) -> proc_macro2::TokenStream {
    match &field.serializer {
        Some(FieldFunction::Generated)
            if component_field_serialization_kind(&field.ty).is_some() =>
        {
            let field_ident = &field.ident;
            let serializer_ident = format_ident!("wxr_serialize_{}_{}", component_id, field_ident);
            quote! { Some(#serializer_ident) }
        }
        Some(FieldFunction::Generated) => quote! { None },
        Some(FieldFunction::Custom(serializer)) => quote! {{
            let serializer: ::wasserxr::scene::component::Serializer = #serializer;
            Some(serializer)
        }},
        None => quote! { None },
    }
}

fn create_deserializer_option(component_id: &str, field: &Field) -> proc_macro2::TokenStream {
    match &field.deserializer {
        Some(FieldFunction::Generated)
            if component_field_serialization_kind(&field.ty).is_some() =>
        {
            let field_ident = &field.ident;
            let deserializer_ident =
                format_ident!("wxr_deserialize_{}_{}", component_id, field_ident);
            quote! { Some(#deserializer_ident) }
        }
        Some(FieldFunction::Generated) => quote! { None },
        Some(FieldFunction::Custom(deserializer)) => quote! {{
            let deserializer: ::wasserxr::scene::component::Deserializer = #deserializer;
            Some(deserializer)
        }},
        None => quote! { None },
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
        .filter(|field| matches!(&field.serializer, Some(FieldFunction::Generated)))
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
    let deserializers = fields
        .iter()
        .filter(|field| matches!(&field.deserializer, Some(FieldFunction::Generated)))
        .filter_map(|field| {
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
