# Learning Rust Macros With `macro_rules!`

This file describes the earlier `macro_rules!` approach that WasserXR used
before the system macro became a proc macro.

The old call shape looked like this:

```rust
system! {
    id = "my_system",
    group_symbol = "WXR_GROUPS_MY_SYSTEM",
    entities = [
        ["Transform", "Mesh"],
        ["Camera"],
    ],
    pub fn my_system(scene: &mut Scene, entities: Vec<Vec<Uuid>>, groups: Vec<usize>) {
    }
}
```

That version was useful as a teaching step because `macro_rules!` is Rust's
simple pattern-based macro system. A `macro_rules!` macro matches token
patterns and emits replacement tokens:

```rust
macro_rules! make_number {
    ($name:ident, $value:expr) => {
        let $name = $value;
    };
}
```

This call:

```rust
make_number!(health, 100);
```

expands to:

```rust
let health = 100;
```

The important pieces are fragment specifiers:

```rust
$name:ident      // an identifier
$value:expr      // an expression
$ty:ty           // a type
$body:block      // a { ... } block
$literal:literal // a string, number, bool, etc.
```

For repeated input, `macro_rules!` uses repetition:

```rust
$( $component:literal ),*
```

That means "zero or more string literals separated by commas." The old system
macro used nested repetition to match groups:

```rust
entities = [
    $( [ $( $component:literal ),* $(,)? ] ),+ $(,)?
],
```

The limitation was name generation. `macro_rules!` can concatenate string
literals with `concat!`, but it cannot turn `"my_system"` into `"MY_SYSTEM"`.
That is why the old macro required an explicit `group_symbol`.

The current implementation uses a proc macro instead. Proc macros can run Rust
code during expansion, so they can compute names like `WXR_GROUPS_MY_SYSTEM`
from the function name directly.
