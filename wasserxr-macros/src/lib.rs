mod component;
mod system;

// `proc_macro::TokenStream` is the compiler-facing token type.
// Attribute macros receive token streams and must return a token stream.
use proc_macro::TokenStream;

// `Error` lets the public macro entrypoints convert generation failures into compile errors.
// `ItemFn` and `ItemStruct` parse the item each attribute is attached to.
// `parse_macro_input!` parses raw compiler tokens into those `syn` structures.
use syn::{Error, ItemFn, ItemStruct, parse_macro_input};

// Rust normally expects function names to be snake_case.
// The user-facing macro name is intentionally `System` to match the issue design.
#[allow(non_snake_case)]
// This marks `System` as an attribute proc macro, usable as `#[System(...)]`.
#[proc_macro_attribute]
// `args` are the tokens inside `#[System(...)]`.
// `item` is the function item the attribute is attached to.
pub fn System(args: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the attribute arguments into the system module's custom argument struct.
    // On parse failure, `parse_macro_input!` returns a compiler error token stream.
    let args = parse_macro_input!(args as system::Args);

    // Parse the annotated item as a Rust function.
    // This intentionally rejects using `#[System]` on structs, modules, etc.
    let item = parse_macro_input!(item as ItemFn);

    // Generate the expanded Rust code.
    system::expand(args, item)
        // If generation returns a `syn::Error`, turn it into `compile_error!`.
        .unwrap_or_else(Error::into_compile_error)
        // Convert `proc_macro2::TokenStream` into the compiler's `proc_macro::TokenStream`.
        .into()
}

// Rust normally expects function names to be snake_case.
// The user-facing macro name is intentionally `Component` to match the issue design.
#[allow(non_snake_case)]
// This marks `Component` as an attribute proc macro, usable as `#[Component]`.
#[proc_macro_attribute]
// `args` are the tokens inside `#[Component(...)]`, which this macro does not use.
// `item` is the struct item the attribute is attached to.
pub fn Component(args: TokenStream, item: TokenStream) -> TokenStream {
    // The component macro deliberately has no arguments, so reject anything inside `#[Component(...)]`.
    if !args.is_empty() {
        // Return a normal Rust compile error at the attribute site.
        return Error::new(
            proc_macro2::Span::call_site(),
            "`Component` does not take arguments",
        )
        .into_compile_error()
        .into();
    }

    // Parse the annotated item as a Rust struct.
    // This intentionally rejects using `#[Component]` on functions, enums, modules, etc.
    let item = parse_macro_input!(item as ItemStruct);

    // Generate the expanded Rust code.
    component::expand(item)
        // If generation returns a `syn::Error`, turn it into `compile_error!`.
        .unwrap_or_else(Error::into_compile_error)
        // Convert `proc_macro2::TokenStream` into the compiler's `proc_macro::TokenStream`.
        .into()
}
