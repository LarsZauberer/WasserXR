# Building the System Proc Macro

This document explains what the `#[System(...)]` macro generates for WasserXR.
For a more general explanation of how proc macros work, see
`educational_proc_macro.md`.

## User-Facing Syntax

A system author writes a normal Rust function and adds the `System` attribute:

```rust
use wasserxr::{Scene, System, Uuid};

#[System(entities = [["Transform", "Mesh"], ["Camera"], ["Window"]])]
pub fn my_system(scene: &mut Scene, entities: Vec<Vec<Uuid>>, groups: Vec<usize>) {
    // system code
}
```

The group count is inferred from the number of inner arrays in `entities`.

## Exported Symbols

WasserXR loads systems by looking up C ABI symbols. For `my_system`, the macro
generates:

```text
WXR_GROUPS_MY_SYSTEM
wxr_select_my_system
wxr_system_my_system
```

The group symbol is computed from the Rust function name by uppercasing it and
adding the `WXR_GROUPS_` prefix.

## Selector

The generated selector receives an entity pointer from the engine, converts the
16 UUID bytes into a `Uuid`, and checks the component groups in order.

If an entity has every component in group 0, the selector returns `0`. If not,
it checks group 1, then group 2, and so on. If no group matches, it returns
`-1`.

This preserves the required rule: if an entity fits multiple groups, the lowest
group wins.

## Runner

The generated runner receives the raw C ABI pointers from `Scene::run_system`.
It copies those pointers into Rust-friendly values:

```rust
Vec<Vec<Uuid>>
Vec<usize>
```

Then it calls the original Rust function:

```rust
my_system(scene, rust_entities, groups);
```

The generated runner does not take ownership of the raw ABI memory.

## Attacher and Detacher

The macro does not generate attacher or detacher symbols. WasserXR already
falls back to no-op defaults when those symbols are missing.
