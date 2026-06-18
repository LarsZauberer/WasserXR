# Building the System Macro

This document explains the `system!` macro step by step. The goal is to turn a normal Rust
function into the C-ABI symbols that WasserXR already knows how to load.

## 1. Start with the Rust function

A system author wants to write normal Rust:

```rust
pub fn my_system(scene: &mut Scene, entities: Vec<Vec<Uuid>>, groups: Vec<usize>) {
    // system code
}
```

`Scene::tick` cannot call that function directly. It calls a function pointer loaded from a
symbol named `wxr_system_my_system`. So the macro keeps the Rust function and adds a wrapper
with the exported symbol name.

## 2. Export the names WasserXR looks up

`System::new` currently looks for these symbols:

```text
wxr_system_my_system
wxr_select_my_system
WXR_GROUPS_MY_SYSTEM
```

The macro uses `#[unsafe(export_name = "...")]` for those names. The Rust helper function can
have a simple private name like `runner`; only the exported symbol needs to match the engine.

The group symbol is passed explicitly:

```rust
group_symbol = "WXR_GROUPS_MY_SYSTEM"
```

That is intentional. A `macro_rules!` macro can paste string literals with `concat!`, but it
cannot reliably turn `my_system` into uppercase `MY_SYSTEM` without adding more machinery.

## 3. Keep generated symbols alive

Generated ABI items may look unused to Rust because the engine finds them later through
`dlsym`. The macro adds small `#[used]` statics that reference the generated symbols. This
tells the compiler and linker that the symbols are intentionally needed.

## 4. Count groups from the selector list

The macro call lists groups once:

```rust
entities = [
    ["Transform", "Mesh"],
    ["Camera"],
    ["Window"],
]
```

The macro counts those entries and exports that number as `WXR_GROUPS_*`. This avoids writing
the same number twice and accidentally letting it drift from the actual selector list.

## 5. Build the selector

The generated selector receives raw pointers:

```rust
unsafe extern "C" fn(scene: *const Scene, entity: *const u8) -> i32
```

The entity pointer points at the 16 bytes of a `Uuid`. The selector copies those bytes back
into a `Uuid`, then checks the groups in order:

```rust
if entity has every component in group 0 {
    return 0;
}
if entity has every component in group 1 {
    return 1;
}
return -1;
```

Checking in list order gives the required behavior: if an entity fits multiple groups, the
lowest group wins.

## 6. Build the runner

The generated runner receives the raw group arrays from `Scene::run_system`:

```rust
unsafe extern "C" fn(
    scene: *mut Scene,
    entities: *const *const *const u8,
    groups: *const usize,
)
```

The macro copies those raw UUID pointers into `Vec<Vec<Uuid>>` and copies the group sizes into
`Vec<usize>`. It does not take ownership of the raw memory. The vectors are only friendly Rust
values for the original system function:

```rust
my_system(&mut *scene, rust_entities, group_sizes);
```

## 7. Final macro call

The finished macro call looks like this:

```rust
system! {
    id = "my_system",
    group_symbol = "WXR_GROUPS_MY_SYSTEM",
    entities = [
        ["Transform", "Mesh"],
        ["Camera"],
        ["Window"],
    ],
    pub fn my_system(scene: &mut Scene, entities: Vec<Vec<Uuid>>, groups: Vec<usize>) {
        // system code
    }
}
```

This is not the same as an attribute macro like `#[System(...)]`. Attribute macros require a
separate proc-macro crate. For this feature, `macro_rules!` keeps the implementation small and
visible enough to learn from.
