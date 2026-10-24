# 1. Syntax & Control Flow

**What this replaces:** Python's dynamic typing + Java's static typing —
Rust is statically typed like Java, but type inference is aggressive
enough that it *feels* like Python most of the time.

## Variables: immutable by default

```rust
let x = 5;          // immutable — cannot reassign x
let mut y = 5;       // mutable — y = 6 is allowed
```

The opposite default from Python (everything reassignable) and Java
(mutable unless `final`). Rust makes you opt *in* to mutability, which
turns "did something change this behind my back" into a compile-time
question instead of a debugging session.

## Shadowing — not the same as mutation

```rust
let x = 5;
let x = x + 1;        // a NEW variable, also named x — not mutating the old one
let x = x.to_string(); // can even change type, since it's a new binding
```

Different from `mut`: shadowing creates a distinct variable that happens
to reuse the name (common for "parse this, then use the parsed form
under the same name"). `mut` requires the type to stay the same;
shadowing doesn't.

## Types: inferred, but always static

| | Python | Java | Rust |
|---|---|---|---|
| Type known at | runtime | compile time | compile time |
| Annotation required? | never (optional hints) | always | rarely — inferred from usage |
| Can a variable change type? | yes | no | no (shadowing aside) |

```rust
let count = 5;          // inferred as i32
let count: u64 = 5;     // explicit when needed (inference can't always guess)
let name = "Ferris";    // inferred as &str
```

Numeric types are explicit and sized: `i32`/`i64`/`u32`/`u64`/`usize`
(signed/unsigned, bit width), `f32`/`f64` (floats), `bool`, `char` (a
Unicode scalar value, 4 bytes — not a byte like Java/C's `char`).
`usize` is the one to know: it's the pointer-width unsigned integer,
used for indexing and lengths (`vec.len()` returns `usize`).

## Functions

```rust
fn add(a: i32, b: i32) -> i32 {
    a + b   // no `return`, no `;` — last expression is the return value
}
```

- Parameter and return types are **always** explicit (no inference for
  function signatures, unlike `let`).
- **No `;` on the last line = implicit return.** This trips people up
  constantly coming from Python/Java, where `return` is mandatory. Add a
  `;` and it becomes a statement returning `()` instead — a very common
  "why won't this compile, the types look right" bug.
- `return` still exists for early returns: `if bad { return -1; }`.

## `if` is an expression, not a statement

```rust
let max = if a > b { a } else { b };   // no ternary operator — this is it
```

Unlike Java's `if`/`else` (statements only, hence `?:` exists
separately), Rust's `if`/`else` produce a value directly — both branches
must produce the same type. This is the same "block evaluates to its
last expression" rule functions use.

## Loops

```rust
for i in 0..5 { }        // 0,1,2,3,4 — exclusive range
for i in 0..=5 { }       // 0,1,2,3,4,5 — inclusive range
for item in &vec { }     // iterate by reference (see ch. 5)

while condition { }

loop {                    // infinite loop — Rust's `while(true)`
    if done { break; }
}

let result = loop {
    if done { break 42; }   // `break` can carry a value out of `loop`
};
```

`for` always iterates something implementing `Iterator` (ranges,
collections, etc.) — there's no C-style `for (i=0; i<n; i++)`. Loops can
be labeled for `break`/`continue` on an outer loop from inside a nested
one: `'outer: for x in .. { for y in .. { break 'outer; } }`.

## Everything is an expression — the theme to internalize

`if`, `match`, `loop`, and `{ }` blocks themselves all produce values.
This is the single biggest *feel* difference from Java (where these are
all statements) — Rust code tends to be built from nested expressions
rather than sequences of statements with temporary variables.

```rust
let description = {
    let tmp = compute_something();
    format!("value: {tmp}")   // no `;` — the block evaluates to this
};
```
