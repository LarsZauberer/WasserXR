// `format_ident!` builds Rust identifiers such as `wxr_create_MyComponent`.
// `quote!` lets us write Rust-looking output code and turn it into tokens.
use quote::{format_ident, quote};

// These `syn` types parse raw tokens into structured Rust syntax.
// That is easier than manually inspecting every punctuation token ourselves.
use syn::{
    // `Error` produces compile errors at the user's macro call site.
    Error,
    // `Fields` lets us distinguish named, tuple, and unit structs.
    Fields,
    // `Ident` stores names like `my_string`.
    Ident,
    // `ItemStruct` parses the struct that has `#[Component]` on it.
    ItemStruct,
    // `Result` is `syn::Result`, the normal result type for parsers/generators.
    Result,
    // `Type` stores a parsed Rust type such as `i32` or `String`.
    Type,
};

// This stores the information the component macro needs for one struct field.
// It is separate from `syn::Field` because we strip marker attributes before
// re-emitting the user's struct, but still need to remember what they meant.
struct Field {
    // The field name, for example `my_string`.
    ident: Ident,
    // The field type, for example `String`.
    ty: Type,
    // Whether the user wrote `#[getter]` on this field.
    has_getter: bool,
    // Whether the user wrote `#[setter]` on this field.
    has_setter: bool,
}

// This function contains the component code-generation logic.
// It takes ownership of the parsed struct because it removes `#[getter]` and `#[setter]`
// before emitting the user's struct back to the compiler.
pub(crate) fn expand(mut item: ItemStruct) -> Result<proc_macro2::TokenStream> {
    // Read the struct name, for example `MyComponent`.
    let component_ident = item.ident.clone();

    // Convert the struct identifier into the component id string used by `Scene::add_component`.
    let component_id = component_ident.to_string();

    // Compute the exported creator symbol, for example `wxr_create_MyComponent`.
    let creator_name = format!("wxr_create_{}", component_id);

    // Compute the exported destroyer symbol, for example `wxr_destroy_MyComponent`.
    let destroyer_name = format!("wxr_destroy_{}", component_id);

    // Compute the exported schema symbol, for example `wxr_schema_MyComponent`.
    let schema_name = format!("wxr_schema_{}", component_id);

    // Build a Rust identifier for the generated creator function.
    let creator_ident = format_ident!("{}", creator_name);

    // Build a Rust identifier for the generated destroyer function.
    let destroyer_ident = format_ident!("{}", destroyer_name);

    // Build a Rust identifier for the generated schema function.
    let schema_ident = format_ident!("{}", schema_name);

    // Collect parsed field information here.
    let mut component_fields = Vec::new();

    // Match only named-field structs because tuple/unit fields do not have schema names.
    let Fields::Named(fields) = &mut item.fields else {
        // Point the compile error at the whole struct if it is not a named-field struct.
        return Err(Error::new_spanned(
            item,
            "`Component` only supports structs with named fields",
        ));
    };

    // Visit every field so we can record and remove `#[getter]` / `#[setter]`.
    for field in fields.named.iter_mut() {
        // Named fields always have an identifier, but keep the error explicit for clarity.
        let Some(field_ident) = field.ident.clone() else {
            return Err(Error::new_spanned(
                field,
                "`Component` only supports named fields",
            ));
        };

        // Track whether marker attributes were present on this field.
        let mut has_getter = false;

        // Track whether marker attributes were present on this field.
        let mut has_setter = false;

        // Preserve non-marker attributes here so they remain on the user's struct.
        let mut kept_attrs = Vec::new();

        // Move every field attribute out, inspect it, and decide whether to keep it.
        for attr in field.attrs.drain(..) {
            // `#[getter]` means generate a getter function and register it in the schema.
            if attr.path().is_ident("getter") {
                has_getter = true;
            // `#[setter]` means generate a setter function and register it in the schema.
            } else if attr.path().is_ident("setter") {
                has_setter = true;
            // Every other attribute belongs to the user's struct and should be preserved.
            } else {
                kept_attrs.push(attr);
            }
        }

        // Put the preserved attributes back on the field.
        // This removes unknown marker attrs so the expanded struct still compiles.
        field.attrs = kept_attrs;

        // Store the field metadata needed for schema/getter/setter generation.
        component_fields.push(Field {
            ident: field_ident,
            ty: field.ty.clone(),
            has_getter,
            has_setter,
        });
    }

    // Generate all getter functions requested by field attributes.
    let getters = component_fields
        .iter()
        // Only fields marked `#[getter]` need getter ABI functions.
        .filter(|field| field.has_getter)
        // Convert each field into a quoted getter function.
        .map(|field| {
            // Borrow the field name for generated field access.
            let field_ident = &field.ident;

            // Borrow the field type for pointer casts.
            let field_ty = &field.ty;

            // Compute the getter symbol name, for example `wxr_get_MyComponent_my_int`.
            let getter_name = format!("wxr_get_{}_{}", component_id, field_ident);

            // Build the Rust identifier for that generated getter function.
            let getter_ident = format_ident!("{}", getter_name);

            // Return generated Rust code for one getter.
            quote! {
                // Export this getter under the symbol name referenced from the schema.
                #[unsafe(export_name = #getter_name)]
                // The generated name can contain the component's uppercase Rust type name.
                #[allow(non_snake_case)]
                pub unsafe extern "C" fn #getter_ident(
                    // The engine passes the component data pointer as opaque C memory.
                    ptr: *const ::std::ffi::c_void,
                ) -> *const ::std::ffi::c_void {
                    // Cast the opaque pointer back to the concrete component type,
                    // borrow the field, and return that field address as an opaque pointer.
                    unsafe {
                        &(*(ptr as *const #component_ident)).#field_ident
                            as *const #field_ty
                            as *const ::std::ffi::c_void
                    }
                }
            }
        });

    // Generate all setter functions requested by field attributes.
    let setters = component_fields
        .iter()
        // Only fields marked `#[setter]` need setter ABI functions.
        .filter(|field| field.has_setter)
        // Convert each field into a quoted setter function.
        .map(|field| {
            // Borrow the field name for generated field access.
            let field_ident = &field.ident;

            // Borrow the field type for pointer casts.
            let field_ty = &field.ty;

            // Compute the setter symbol name, for example `wxr_set_MyComponent_my_string`.
            let setter_name = format!("wxr_set_{}_{}", component_id, field_ident);

            // Build the Rust identifier for that generated setter function.
            let setter_ident = format_ident!("{}", setter_name);

            // Return generated Rust code for one setter.
            quote! {
                // Export this setter under the symbol name referenced from the schema.
                #[unsafe(export_name = #setter_name)]
                // The generated name can contain the component's uppercase Rust type name.
                #[allow(non_snake_case)]
                pub unsafe extern "C" fn #setter_ident(
                    // The engine passes the component data pointer as opaque mutable C memory.
                    ptr: *mut ::std::ffi::c_void,
                    // `data` points to a borrowed value owned by the caller.
                    data: *const ::std::ffi::c_void,
                ) {
                    // Cast both opaque pointers to concrete Rust types.
                    // Clone from `data` so the setter copies instead of taking ownership.
                    unsafe {
                        (*(ptr as *mut #component_ident)).#field_ident =
                            (*(data as *const #field_ty)).clone();
                    }
                }
            }
        });

    // Generate one schema `add_field` call per struct field.
    let schema_fields = component_fields.iter().map(|field| {
        // Borrow the field name for schema registration and generated symbol names.
        let field_ident = &field.ident;

        // Convert the field identifier into the string stored in the schema.
        let field_name = field_ident.to_string();

        // Infer the WasserXR field type from the Rust field type.
        let field_type = component_field_type(&field.ty);

        // If the field has a getter, pass `Some(getter_fn)` to the schema.
        // Otherwise pass `None`, meaning `Scene::get` will report `FieldNoGetter`.
        let getter = if field.has_getter {
            let getter_ident = format_ident!("wxr_get_{}_{}", component_id, field_ident);
            quote! { Some(#getter_ident) }
        } else {
            quote! { None }
        };

        // If the field has a setter, pass `Some(setter_fn)` to the schema.
        // Otherwise pass `None`, meaning `Scene::set` will report `FieldNoSetter`.
        let setter = if field.has_setter {
            let setter_ident = format_ident!("wxr_set_{}_{}", component_id, field_ident);
            quote! { Some(#setter_ident) }
        } else {
            quote! { None }
        };

        // Return generated Rust code for one schema field registration.
        quote! {
            // Add this field to the component schema even if getter/setter are absent.
            (*schema).add_field(
                #field_name.to_owned(),
                #field_type,
                #getter,
                #setter,
            );
        }
    });

    // Return the final generated Rust code.
    // Everything inside `quote!` is compiled in the user's crate after expansion.
    Ok(quote! {
        // Re-emit the user's struct, with `#[getter]` and `#[setter]` removed from fields.
        #item

        // Export the component creator under the symbol name `Component::new` searches for.
        #[unsafe(export_name = #creator_name)]
        // The generated name can contain the component's uppercase Rust type name.
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn #creator_ident() -> *mut ::std::ffi::c_void {
            // Allocate a default component value on the heap.
            // `Box::into_raw` transfers ownership to the engine-side component wrapper.
            Box::into_raw(Box::new(#component_ident::default())) as *mut ::std::ffi::c_void
        }

        // Export the component destroyer under the symbol name `Component::new` searches for.
        #[unsafe(export_name = #destroyer_name)]
        // The generated name can contain the component's uppercase Rust type name.
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn #destroyer_ident(ptr: *mut ::std::ffi::c_void) {
            // Cast the opaque pointer back to the heap allocation created above.
            // `Box::from_raw` takes ownership back so dropping the box frees the component.
            unsafe {
                drop(Box::from_raw(ptr as *mut #component_ident));
            }
        }

        // Export the schema creator under the symbol name `Component::new` searches for.
        #[unsafe(export_name = #schema_name)]
        // The generated name can contain the component's uppercase Rust type name.
        #[allow(non_snake_case)]
        pub unsafe extern "C" fn #schema_ident(
            // The engine passes an empty schema for this function to fill.
            schema: *mut ::wasserxr::scene::component::Schema,
        ) {
            // Register every struct field with inferred type and optional getter/setter hooks.
            unsafe {
                #(#schema_fields)*
            }
        }

        // Emit all generated getter functions after the schema function.
        #(#getters)*

        // Emit all generated setter functions after the schema function.
        #(#setters)*
    })
}

// Infer the WasserXR `FieldType` for a Rust field type.
// This is intentionally simple: unknown or complex types fall back to `Blob`.
fn component_field_type(ty: &Type) -> proc_macro2::TokenStream {
    // Try to interpret the type as a path like `i32`, `String`, or `std::string::String`.
    let Type::Path(path) = ty else {
        // References, tuples, arrays, and other complex syntax use Blob.
        return quote! { ::wasserxr::scene::component::FieldType::Blob };
    };

    // Read the final path segment.
    // For `std::string::String`, the final segment is `String`.
    let Some(segment) = path.path.segments.last() else {
        // Empty paths are unusual, but Blob is the conservative fallback.
        return quote! { ::wasserxr::scene::component::FieldType::Blob };
    };

    // Convert the segment identifier to text for simple matching.
    let type_name = segment.ident.to_string();

    // Match known Rust primitive/string names to WasserXR field hints.
    match type_name.as_str() {
        // WasserXR currently has one integer-ish field hint, so all integer widths use Long.
        "i8" | "i16" | "i32" | "i64" | "i128" | "isize" | "u8" | "u16" | "u32" | "u64" | "u128"
        | "usize" => quote! { ::wasserxr::scene::component::FieldType::Long },
        // Both Rust float widths map to WasserXR's Float hint.
        "f32" | "f64" => quote! { ::wasserxr::scene::component::FieldType::Float },
        // Rust `char` maps directly to WasserXR's Char hint.
        "char" => quote! { ::wasserxr::scene::component::FieldType::Char },
        // Owned Rust strings map to WasserXR's String hint.
        "String" => quote! { ::wasserxr::scene::component::FieldType::String },
        // Everything else is opaque to the schema.
        _ => quote! { ::wasserxr::scene::component::FieldType::Blob },
    }
}
