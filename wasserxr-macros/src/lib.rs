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
/// #[component(no_schema)]
/// #[derive(Default)]
/// struct MyComponent {
///     value: i32,
///     #[getter(custom_name_getter)]
///     #[mutable]
///     name: String,
///     #[none]
///     internal: i32,
/// }
/// ```
///
/// The macro exports create, destroy, and schema functions for the component.
/// Use `#[component(no_schema)]` to skip schema generation and provide a custom
/// `wxr_schema_<Component>` function yourself.
/// Fields without field function attributes get generated getter, serializer,
/// and deserializer functions by default. Use `#[mutable]` to allow mutable
/// references through `Scene::query_mut`. If at least one field function
/// attribute is present, only the requested functions are generated. Field
/// function attributes can also take a custom function path, for example
/// `#[getter(my_getter)]`. Use `#[none]` to register a field without generated
/// field functions.
#[proc_macro_attribute]
pub fn component(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as component::Args);
    let item = parse_macro_input!(item as ItemStruct);

    component::expand(args, item)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
