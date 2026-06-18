# Learning Rust Macros With `system!`

This document explains how to write a Rust macro, using WasserXR's `system!`
macro as the example.

It does not focus on WasserXR's runtime behavior. Instead, it focuses on the
syntax and thinking process behind a macro like this:

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

## 1. What Is a Macro?

A macro is code that writes code.

A normal Rust function runs at runtime:

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

A macro runs while Rust is compiling your program. It receives Rust tokens as
input and expands them into different Rust tokens.

For example, this:

```rust
println!("Hello {}", name);
```

is a macro call. The `println!` macro expands into lower-level formatting and
output code before the program is compiled.

The `!` is the visual sign that you are calling a macro:

```rust
println!(...);
vec![1, 2, 3];
system! { ... }
```

## 2. Why Use a Macro Here?

Without a macro, a WasserXR system needs several pieces:

```rust
pub fn my_system(...) {
    // friendly Rust code
}

#[unsafe(export_name = "WXR_GROUPS_MY_SYSTEM")]
static GROUPS: usize = 3;

#[unsafe(export_name = "wxr_select_my_system")]
unsafe extern "C" fn selector(...) -> i32 {
    // select a group
}

#[unsafe(export_name = "wxr_system_my_system")]
unsafe extern "C" fn runner(...) {
    // convert raw C pointers into Rust values
    my_system(...);
}
```

Most of this code is repetitive. The system author should only need to write:

- the system id,
- the component groups,
- the Rust function body.

That is a good use case for a macro.

## 3. `macro_rules!`

The macro in this project is written with `macro_rules!`:

```rust
macro_rules! system {
    (...) => {
        ...
    };
}
```

`macro_rules!` is Rust's built-in pattern-based macro system.

It works like this:

1. You describe an input pattern.
2. You name parts of that pattern.
3. You write output code using those named parts.

The basic shape is:

```rust
macro_rules! name {
    (input pattern) => {
        output code
    };
}
```

The macro name here is `system`:

```rust
macro_rules! system {
    ...
}
```

That lets users call:

```rust
system! {
    ...
}
```

## 4. A Tiny Macro First

Before looking at `system!`, start with a tiny macro:

```rust
macro_rules! make_answer {
    () => {
        let answer = 42;
    };
}
```

This macro accepts no input. The empty `()` means "match an empty macro call".

Use it like this:

```rust
fn main() {
    make_answer!();
    println!("{answer}");
}
```

The compiler expands it as if you wrote:

```rust
fn main() {
    let answer = 42;
    println!("{answer}");
}
```

This is the core idea: the macro call is replaced by the output tokens.

## 5. Capturing One Value

Macros become useful when they capture input:

```rust
macro_rules! make_number {
    ($name:ident, $value:expr) => {
        let $name = $value;
    };
}
```

Use it like this:

```rust
make_number!(health, 100);
```

It expands to:

```rust
let health = 100;
```

The important syntax is:

```rust
$name:ident
$value:expr
```

`$name` and `$value` are metavariables. They are variables inside the macro.

The part after `:` is the fragment type:

- `ident` means an identifier, like `health` or `my_system`.
- `expr` means an expression, like `100`, `1 + 2`, or `Scene::new()`.

## 6. Common Fragment Types

`macro_rules!` does not capture normal Rust types like `String` or `usize`.
It captures pieces of Rust syntax.

Useful fragment types include:

```rust
$name:ident    // an identifier: my_system
$value:expr    // an expression: 1 + 2
$ty:ty         // a type: Vec<Uuid>
$body:block    // a block: { ... }
$vis:vis       // visibility: pub, pub(crate), or nothing
$literal:literal // a literal: "hello", 3, true
$tokens:tt     // one token tree: very flexible
```

The `system!` macro uses several of these:

```rust
$id:literal
$group_symbol:literal
$vis:vis
$name:ident
$scene_ty:ty
$entity_ty:ty
$body:block
```

## 7. Matching the `system!` Input

The current macro starts like this:

```rust
macro_rules! system {
    (
        id = $id:literal,
        group_symbol = $group_symbol:literal,
        entities = [
            $( [ $( $component:literal ),* $(,)? ] ),+ $(,)?
        ],
        $vis:vis fn $name:ident(
            $scene_arg:ident : &mut $scene_ty:ty,
            $entities_arg:ident : Vec<Vec<$entity_ty:ty>>,
            $groups_arg:ident : Vec<usize> $(,)?
        ) $body:block
    ) => {
        ...
    };
}
```

This looks intimidating, so split it into parts.

## 8. Matching Literal Keywords

This part:

```rust
id = $id:literal,
```

matches input like:

```rust
id = "my_system",
```

The tokens `id`, `=`, and `,` must appear exactly.

Only `$id:literal` is captured. In this example, `$id` becomes:

```rust
"my_system"
```

The same idea is used here:

```rust
group_symbol = $group_symbol:literal,
```

That matches:

```rust
group_symbol = "WXR_GROUPS_MY_SYSTEM",
```

## 9. Matching the Function Signature

This part matches the Rust function:

```rust
$vis:vis fn $name:ident(
    $scene_arg:ident : &mut $scene_ty:ty,
    $entities_arg:ident : Vec<Vec<$entity_ty:ty>>,
    $groups_arg:ident : Vec<usize> $(,)?
) $body:block
```

It matches input like:

```rust
pub fn my_system(
    scene: &mut Scene,
    entities: Vec<Vec<Uuid>>,
    groups: Vec<usize>,
) {
    // body
}
```

The captured pieces are:

```rust
$vis        // pub
$name       // my_system
$scene_arg  // scene
$scene_ty   // Scene
$entities_arg // entities
$entity_ty  // Uuid
$groups_arg // groups
$body       // { ... }
```

The macro can then put these pieces into generated code.

## 10. Re-Emitting the Function

The macro first expands back into the user's original Rust function:

```rust
$vis fn $name(
    $scene_arg: &mut $scene_ty,
    $entities_arg: Vec<Vec<$entity_ty>>,
    $groups_arg: Vec<usize>,
) $body
```

If the user wrote:

```rust
pub fn my_system(scene: &mut Scene, entities: Vec<Vec<Uuid>>, groups: Vec<usize>) {
    println!("run");
}
```

then the macro emits:

```rust
pub fn my_system(scene: &mut Scene, entities: Vec<Vec<Uuid>>, groups: Vec<usize>) {
    println!("run");
}
```

At this point the macro has done nothing magical. It captured the function and
printed it back out.

That is a good way to develop macros: start by matching input and re-emitting
almost the same code.

## 11. Adding Generated Code

After re-emitting the function, the macro adds extra generated items:

```rust
const _: () = {
    // generated static
    // generated selector
    // generated runner
};
```

This `const _: () = { ... };` is a useful trick.

It creates a small anonymous scope for generated helper names. Inside that
scope, the macro can use simple names like `GROUPS`, `selector`, and `runner`
without colliding with other systems that also use the macro.

For example, two macro calls can both generate a private function named
`runner`, because each one is inside its own anonymous `const` block.

## 12. Repetition Syntax

The hardest-looking part of the macro is the component group matcher:

```rust
entities = [
    $( [ $( $component:literal ),* $(,)? ] ),+ $(,)?
],
```

This matches input like:

```rust
entities = [
    ["Transform", "Mesh"],
    ["Camera"],
    ["Window"],
],
```

Rust macro repetition uses this shape:

```rust
$( pattern ),*
```

That means:

- match `pattern`,
- separated by commas,
- zero or more times.

There is also:

```rust
$( pattern ),+
```

That means one or more times.

So this inner part:

```rust
$( $component:literal ),*
```

matches component names inside one group:

```rust
"Transform", "Mesh"
```

This outer part:

```rust
$( [ $( $component:literal ),* $(,)? ] ),+
```

matches one or more groups:

```rust
["Transform", "Mesh"],
["Camera"],
["Window"]
```

The small part:

```rust
$(,)?
```

means "an optional trailing comma".

That lets both of these compile:

```rust
["Camera"]
```

and:

```rust
["Camera"],
```

## 13. Using Repeated Values in Output

The macro later emits this:

```rust
let selectors: &[&[&str]] = &[
    $( &[ $( $component ),* ] ),+
];
```

This uses the same repetition structure from the matcher.

If the input was:

```rust
entities = [
    ["Transform", "Mesh"],
    ["Camera"],
]
```

then the output becomes:

```rust
let selectors: &[&[&str]] = &[
    &["Transform", "Mesh"],
    &["Camera"],
];
```

This is an important rule: when you capture repeated macro input, you usually
need to repeat it in the output with a matching `$( ... )*` or `$( ... )+`.

## 14. Helper Rules

The macro has extra rules:

```rust
(@count_groups $( $group:tt ),+) => {
    <[()]>::len(&[ $( $crate::system!(@unit $group) ),+ ])
};

(@unit $group:tt) => {
    ()
};
```

These are not meant to be called by users. They are helper rules.

The `@count_groups` and `@unit` names are just tokens chosen by convention.
They make it unlikely that a normal user call conflicts with the helper call.

This call:

```rust
$crate::system!(@count_groups ["Transform"] ["Camera"])
```

would count how many groups were passed.

The trick is that every group becomes `()`:

```rust
[(), ()]
```

Then Rust asks for the length of that array:

```rust
<[()]>::len(&[(), ()])
```

So the count is known at compile time.

## 15. `$crate`

Inside the macro you see paths like:

```rust
$crate::scene::Scene
$crate::system!(@uuid_from_ptr entity)
$crate::r#macro::private::Uuid
```

`$crate` means "the crate where this macro was defined".

This matters because a macro may be called from another crate in the future.
If the macro wrote:

```rust
crate::scene::Scene
```

then `crate` would mean the caller's crate, not necessarily WasserXR.

Using `$crate` makes the macro more robust.

## 16. `#[macro_export]`

The macro has:

```rust
#[macro_export]
macro_rules! system {
    ...
}
```

`#[macro_export]` exports the macro from the crate root.

That means users call it as:

```rust
wasserxr::system! { ... }
```

or, inside this crate's tests:

```rust
crate::system! { ... }
```

Without `#[macro_export]`, the macro would be more local to the module where it
is defined.

## 17. `concat!`

The macro uses:

```rust
#[unsafe(export_name = concat!("wxr_system_", $id))]
```

`concat!` is also a macro. It joins string literals at compile time.

If `$id` is:

```rust
"my_system"
```

then this:

```rust
concat!("wxr_system_", $id)
```

becomes:

```rust
"wxr_system_my_system"
```

This works because both parts are string literals.

## 18. Why `group_symbol` Is Still Explicit

The user asked whether the macro can derive:

```rust
WXR_GROUPS_MY_SYSTEM
```

from:

```rust
id = "my_system"
```

Conceptually, yes:

```text
"WXR_GROUPS_" + id.uppercase()
```

But `macro_rules!` cannot uppercase a string literal.

It can concatenate literals:

```rust
concat!("wxr_system_", "my_system")
```

But it cannot transform `"my_system"` into `"MY_SYSTEM"`.

Also, attributes like this:

```rust
#[unsafe(export_name = "...")]
```

need a literal string after macro expansion. They cannot use a runtime
expression like:

```rust
id.to_uppercase()
```

So with plain `macro_rules!`, the simple solution is to ask for the group symbol
explicitly:

```rust
group_symbol = "WXR_GROUPS_MY_SYSTEM"
```

## 19. What Is `paste`?

`paste` is a small Rust crate that helps macros build identifiers and names from
pieces.

For example, with `paste`, a macro can do things like:

```rust
paste::paste! {
    fn [<wxr_system_ my_system>]() {
        // ...
    }
}
```

It can also transform case in some situations, for example turning an
identifier into uppercase.

That is why `paste` came up in the `group_symbol` discussion. If the macro used
an identifier instead of a string:

```rust
id = my_system
```

then `paste` could help build names based on `my_system`.

The tradeoff is that `paste` is an extra dependency. For this issue, we kept the
macro dependency-free and beginner-readable.

## 20. What Is a Proc Macro?

A proc macro means "procedural macro".

It is a more powerful kind of Rust macro. Instead of just matching patterns like
`macro_rules!`, a proc macro receives token streams and can run Rust code to
produce new token streams.

There are three common kinds:

```rust
#[derive(MyDerive)]
struct Thing;
```

```rust
#[my_attribute(...)]
fn thing() {}
```

```rust
my_macro!(...);
```

The syntax from the Linear issue looked like an attribute macro:

```rust
#[System(groups=3, entities=[["Transform", "Mesh"], ["Camera"]])]
pub fn my_system(...) {
    ...
}
```

That would require a proc macro attribute.

Proc macros are powerful, but they have costs:

- they must live in a separate `proc-macro` crate,
- they usually need parsing libraries like `syn`,
- they usually need code-generation helpers like `quote`,
- they are harder to explain than `macro_rules!`.

For WasserXR's first system macro, `macro_rules!` is simpler.

## 21. `prod_macro` vs `proc_macro`

The correct term is `proc_macro`, short for procedural macro.

If you see `prod_macro`, that is almost certainly a typo.

Rust has a standard library crate named:

```rust
proc_macro
```

That crate is used when implementing procedural macros.

## 22. How I Would Develop This Macro Step by Step

Do not start with the full macro. Build it in small pieces.

### Step 1: Match only the function

Start with:

```rust
macro_rules! system {
    (
        $vis:vis fn $name:ident(
            $scene_arg:ident : &mut $scene_ty:ty,
            $entities_arg:ident : Vec<Vec<$entity_ty:ty>>,
            $groups_arg:ident : Vec<usize>
        ) $body:block
    ) => {
        $vis fn $name(
            $scene_arg: &mut $scene_ty,
            $entities_arg: Vec<Vec<$entity_ty>>,
            $groups_arg: Vec<usize>,
        ) $body
    };
}
```

Then test that this compiles:

```rust
system! {
    pub fn my_system(scene: &mut Scene, entities: Vec<Vec<Uuid>>, groups: Vec<usize>) {
    }
}
```

This proves that you can match and re-emit the function.

### Step 2: Add the id

Add this to the pattern:

```rust
id = $id:literal,
```

Now the call becomes:

```rust
system! {
    id = "my_system",
    pub fn my_system(...) {
    }
}
```

Then use `$id` in one generated symbol:

```rust
#[unsafe(export_name = concat!("wxr_system_", $id))]
unsafe extern "C" fn runner(...) {
}
```

### Step 3: Add groups

Add:

```rust
entities = [
    $( [ $( $component:literal ),* $(,)? ] ),+ $(,)?
],
```

At first, do not implement selection logic. Just turn the captured components
into a Rust value:

```rust
let selectors: &[&[&str]] = &[
    $( &[ $( $component ),* ] ),+
];
```

This proves the repetition syntax is correct.

### Step 4: Count groups

Add helper rules:

```rust
(@count_groups $( $group:tt ),+) => {
    <[()]>::len(&[ $( $crate::system!(@unit $group) ),+ ])
};

(@unit $group:tt) => {
    ()
};
```

Then use them:

```rust
static GROUPS: usize = $crate::system!(@count_groups $( [ $( $component ),* ] ),+);
```

This lets the macro derive the number of groups from the input list.

### Step 5: Add the selector

Generate the selector function:

```rust
#[unsafe(export_name = concat!("wxr_select_", $id))]
unsafe extern "C" fn selector(scene: *const Scene, entity: *const u8) -> i32 {
    ...
}
```

Use the captured component list to check each group in order.

### Step 6: Add the runner

Generate the runner function:

```rust
#[unsafe(export_name = concat!("wxr_system_", $id))]
unsafe extern "C" fn runner(...) {
    ...
}
```

At this point, the macro does the full job:

- match user-friendly input,
- re-emit the Rust function,
- generate WasserXR ABI symbols,
- call the Rust function from the ABI runner.

## 23. Rules of Thumb

When writing `macro_rules!` macros:

- Start with the smallest possible input pattern.
- Re-emit captured input before adding generated code.
- Use fragment types deliberately: `ident`, `ty`, `expr`, `literal`, `block`.
- Use repetition only after the simple case works.
- Use helper rules for counting or repeated transformations.
- Use `$crate` for paths back into the defining crate.
- Prefer clear macro input over clever macro input.
- Reach for `paste` only when you need generated identifiers or case conversion.
- Reach for proc macros only when `macro_rules!` becomes too awkward.

For this WasserXR macro, `macro_rules!` is enough because the desired input is
structured and regular. A proc macro would be more flexible, but also much more
complex.
