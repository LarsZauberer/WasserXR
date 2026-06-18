# Learning Proc Macros With `#[System]`

This document explains how the WasserXR `#[System(...)]` proc macro is built.
It focuses on proc-macro mechanics rather than engine runtime behavior.

## 1. What Is a Proc Macro?

A proc macro is Rust code that runs during compilation and returns Rust code.

The input and output are token streams:

```rust
proc_macro::TokenStream
```

A token stream is roughly "Rust syntax as data." The compiler gives the macro
some tokens, the macro inspects them, and then the macro returns new tokens for
the compiler to compile.

## 2. Why Proc Macros Need a Separate Crate

Rust requires proc macros to live in a crate marked like this:

```toml
[lib]
proc-macro = true
```

That is why WasserXR now has a `wasserxr-macros` crate. The main `wasserxr`
crate depends on it and re-exports the macro:

```rust
pub use wasserxr_macros::System;
```

The proc macro cannot live directly in the main engine crate because the engine
crate builds as `rlib` and `staticlib`, not as a proc-macro crate.

## 3. Attribute Macro Shape

The system macro is an attribute macro:

```rust
#[System(entities = [["Transform", "Mesh"], ["Camera"]])]
pub fn my_system(scene: &mut Scene, entities: Vec<Vec<Uuid>>, groups: Vec<usize>) {
}
```

An attribute macro receives two token streams:

```rust
#[proc_macro_attribute]
pub fn System(args: TokenStream, item: TokenStream) -> TokenStream {
    ...
}
```

`args` contains the tokens inside the attribute:

```rust
entities = [["Transform", "Mesh"], ["Camera"]]
```

`item` contains the item the attribute is attached to:

```rust
pub fn my_system(...) {
}
```

## 4. `syn`: Parsing Tokens Into Rust Structures

Working with raw tokens is tedious. The `syn` crate parses tokens into Rust
data structures.

The function item is parsed as:

```rust
let item = parse_macro_input!(item as syn::ItemFn);
```

`ItemFn` gives access to the function name, signature, visibility, and body:

```rust
let function_name = item.sig.ident.clone();
```

The attribute arguments are parsed with a small custom parser:

```rust
struct SystemArgs {
    entities: Vec<Vec<LitStr>>,
}
```

That parser expects exactly:

```rust
entities = [[...], [...]]
```

and stores each component name as a `LitStr`.

## 5. Inferring the Group Count

The attribute does not need a separate `groups` argument:

```rust
#[System(entities = [["Transform"], ["Camera"]])]
```

The macro can count the inner arrays:

```rust
let group_count = args.entities.len();
```

That value becomes the exported group static:

```rust
static WXR_GROUPS_MY_SYSTEM: usize = 2;
```

This avoids writing the same information twice.

## 6. Computing Symbol Names

Because this is a proc macro, normal Rust string code can run during expansion:

```rust
let system_id = function_name.to_string();
let groups_name = format!("WXR_GROUPS_{}", system_id.to_uppercase());
let selector_name = format!("wxr_select_{}", system_id);
let runner_name = format!("wxr_system_{}", system_id);
```

This is the main advantage over the old `macro_rules!` version. The old macro
could not uppercase `"my_system"` into `"MY_SYSTEM"` by itself.

## 7. `quote`: Generating Rust Code

The `quote` crate turns Rust-like syntax back into tokens.

The proc macro returns code shaped like this:

```rust
quote! {
    #item

    #[unsafe(export_name = #groups_name)]
    static #groups_ident: usize = #group_count;

    #[unsafe(export_name = #selector_name)]
    unsafe extern "C" fn #selector_ident(...) -> i32 {
        ...
    }

    #[unsafe(export_name = #runner_name)]
    unsafe extern "C" fn #runner_ident(...) {
        ...
    }
}
```

The `#name` syntax inserts a Rust value into the generated tokens.

For example:

```rust
#item
```

puts the original user function back into the output.

## 8. Generated Selector

The selector converts the raw entity pointer into a `Uuid`, then checks the
component groups in order:

```rust
let selectors: &[&[&str]] = &[
    &["Transform", "Mesh"],
    &["Camera"],
];
```

The first matching group index is returned. If no group matches, the selector
returns `-1`.

## 9. Generated Runner

The runner converts the engine's raw ABI arrays into friendly Rust values:

```rust
Vec<Vec<Uuid>>
Vec<usize>
```

Then it calls the original function:

```rust
my_system(scene, rust_entities, groups);
```

The runner copies UUID values out of the raw pointers. It does not take
ownership of the ABI memory.

## 10. Why `paste` Is Not Needed

`paste` is useful for `macro_rules!` macros that need to build identifiers from
pieces.

In a proc macro, we can use normal Rust:

```rust
format_ident!("wxr_system_{}", system_id)
```

and:

```rust
system_id.to_uppercase()
```

So the proc macro does not need `paste`.

## 11. Development Strategy

Build proc macros in small steps:

1. Parse the attribute arguments.
2. Parse the function item.
3. Return only the original function with `quote! { #item }`.
4. Add one generated symbol and test it.
5. Add selector generation.
6. Add runner generation.
7. Add tests that use the public syntax.

This keeps the macro understandable and makes compiler errors easier to isolate.
