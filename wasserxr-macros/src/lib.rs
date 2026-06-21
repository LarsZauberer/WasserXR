mod component;
mod system;

use proc_macro::TokenStream;
use syn::{Error, ItemFn, ItemStruct, parse_macro_input};

/// Turns a Rust system attach function into the C ABI function WasserXR needs.
///
/// Use it on a function with this shape:
///
/// ```ignore
/// #[attacher(my_system)]
/// fn attach_my_system(scene: &mut wasserxr::scene::Scene) {
///     // ...
/// }
/// ```
///
/// The macro exports `wxr_attach_<system>`.
#[proc_macro_attribute]
pub fn attacher(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as system::LifecycleArgs);
    let item = parse_macro_input!(item as ItemFn);

    system::expand_attacher(args, item)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Turns a Rust system detach function into the C ABI function WasserXR needs.
///
/// Use it on a function with this shape:
///
/// ```ignore
/// #[detacher(my_system)]
/// fn detach_my_system(scene: &mut wasserxr::scene::Scene) {
///     // ...
/// }
/// ```
///
/// The macro exports `wxr_detach_<system>`.
#[proc_macro_attribute]
pub fn detacher(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as system::LifecycleArgs);
    let item = parse_macro_input!(item as ItemFn);

    system::expand_detacher(args, item)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Turns a Rust system function into the C ABI functions WasserXR needs.
///
/// Use it on a function with this shape:
///
/// ```ignore
/// #[system(entities = [["Transform", "Mesh"], ["Camera"]])]
/// fn render(scene: &mut wasserxr::scene::Scene, entities: Vec<Vec<uuid::Uuid>>) {
///     // ...
/// }
/// ```
///
/// The macro exports `WXR_GROUPS_<SYSTEM>`, `wxr_select_<system>`, and
/// `wxr_system_<system>`. The first matching entity group wins.
#[proc_macro_attribute]
pub fn system(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as system::Args);
    let item = parse_macro_input!(item as ItemFn);

    system::expand(args, item)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Turns a Rust component struct into the C ABI functions WasserXR needs.
///
/// Use it on a named-field struct that implements `Default`:
///
/// ```ignore
/// #[component]
/// #[derive(Default)]
/// struct MyComponent {
///     value: i32,
///     #[getter]
///     #[setter]
///     name: String,
///     #[none]
///     internal: i32,
/// }
/// ```
///
/// The macro exports create, destroy, and schema functions for the component.
/// Fields without field function attributes get generated getter, setter,
/// mover, taker, serializer, and deserializer functions by default. If at
/// least one field function attribute is present, only the requested functions
/// are generated. Use `#[none]` to register a field without generated field
/// functions.
#[proc_macro_attribute]
pub fn component(args: TokenStream, item: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return Error::new(
            proc_macro2::Span::call_site(),
            "`component` does not take arguments",
        )
        .into_compile_error()
        .into();
    }

    let item = parse_macro_input!(item as ItemStruct);

    component::expand(item)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
