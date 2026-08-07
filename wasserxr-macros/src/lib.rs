//! Procedural macros that generate WasserXR's C ABI plugin bindings.
//!
//! These macros let plugin authors write normal Rust systems, components, and
//! asset types while generating callbacks referenced by an explicit plugin
//! descriptor.

mod asset;
mod component;
mod method;
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
/// The macro generates a public `wxr_attach_<system>` callback for a system
/// descriptor to reference.
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
/// The macro generates a public `wxr_detach_<system>` callback for a system
/// descriptor to reference.
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
/// #[system]
/// fn render(scene: &mut wasserxr::scene::Scene, delta: f32, entities: Vec<Vec<uuid::Uuid>>) {
///     // Entity groups are declared in WXRSystemDescriptor and preserve order.
/// }
/// ```
///
/// The macro generates a public `wxr_system_<system>` callback. Every entity
/// group declared in the descriptor is populated independently.
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
/// Use it on a named-field struct:
///
/// ```ignore
/// #[component]
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
/// The macro generates a destroyer and any requested field callbacks. Component
/// fields, their types, mutability, and callback pointers are declared in the
/// plugin descriptor.
/// Fields without field function attributes get generated getter, serializer,
/// and deserializer functions by default. `#[mutable]` ensures a getter is
/// generated; the descriptor's `mutable` flag controls `Scene::query_mut`. If
/// at least one field function attribute is present, only the requested
/// functions are generated. Field function attributes can also take a custom
/// function path, for example `#[getter(my_getter)]`. Use `#[none]` to generate
/// no callbacks for a field.
/// Generated serializers for complex fields use serde through
/// bincode, so those field types must implement serde's serialize and
/// deserialize traits. Structural or computed fields can be declared directly
/// in the descriptor with custom callbacks.
#[proc_macro_attribute]
pub fn component(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as component::Args);
    let item = parse_macro_input!(item as ItemStruct);

    component::expand(args, item)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Turns a Rust function into a WasserXR component method callback.
///
/// Use it on a function with this shape:
///
/// ```ignore
/// #[method(Component)]
/// fn my_method_name(
///     scene: &mut Scene,
///     component: &mut Component,
///     force: &mut f32,
///     iterations: &mut usize,
/// ) -> Result<*mut c_void, i32> {
///     // ...
/// }
/// ```
///
/// The macro generates `wxr_method_<Component>_<my_method_name>`. The first
/// parameter must be exactly `&mut Scene`, the second a mutable reference to the
/// component type named in the attribute, and every remaining parameter a
/// argument is `&mut T` or `Option<&mut T>` with a simple identifier. The
/// descriptor declares each argument's name, type, and nullability, and the
/// host validates and orders arguments before calling the generated callback.
/// The return type must be exactly `Result<*mut c_void, i32>`.
#[proc_macro_attribute]
pub fn method(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as method::Args);
    let item = parse_macro_input!(item as ItemFn);

    method::expand(args, item)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Wraps `fn create(scene: &mut Scene) -> Option<Component>` as a component creator.
///
/// The macro generates `wxr_create_<Component>` and maps `None` to a null pointer.
#[proc_macro_attribute]
pub fn component_creator(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as component::CreatorArgs);
    let item = parse_macro_input!(item as ItemFn);

    component::expand_component_creator(args, item)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Turns a Rust asset struct into the C ABI functions WasserXR needs.
///
/// The macro generates destroy and getter callbacks for the asset type. Every
/// named field gets a getter unless it has `#[none]`; the plugin descriptor
/// decides which generated callbacks form the public asset schema.
#[proc_macro_attribute]
pub fn asset_type(args: TokenStream, item: TokenStream) -> TokenStream {
    if !args.is_empty() {
        return Error::new(
            proc_macro2::Span::call_site(),
            "`asset_type` does not support arguments",
        )
        .into_compile_error()
        .into();
    }

    let item = parse_macro_input!(item as ItemStruct);

    asset::expand_asset_type(item)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}

/// Wraps `fn create(scene: &mut Scene, data: &str) -> Option<AssetType>` as an asset creator.
///
/// The macro generates `wxr_asset_create_<AssetType>` and maps `None` or
/// invalid C strings to a null pointer.
#[proc_macro_attribute]
pub fn asset_type_creator(args: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as asset::CreatorArgs);
    let item = parse_macro_input!(item as ItemFn);

    asset::expand_asset_type_creator(args, item)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
