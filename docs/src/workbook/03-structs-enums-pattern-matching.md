# 3. Structs, Enums & Pattern Matching

**What this replaces:** Python's classes/dataclasses and Java's classes
for structs; Python doesn't really have Rust's enums (closest is
`Enum`/tagged unions via `Union` types, but without the compiler
enforcement); Java's `switch` for `match`, but `match` is far stricter
and more powerful. See [Traits](../foundation/traits.md) and
[Structs & impl](../foundation/structs-and-methods.md) for the deeper
dive on attaching methods, `Self`, and `&self` vs `&mut self` — this
chapter is about the data shapes themselves and matching on them.

## Structs — three flavors

```rust
struct Point { x: f64, y: f64 }              // named fields — most common
struct Pair(i32, i32);                        // tuple struct — fields by position
struct Marker;                                 // unit struct — no fields at all
```

```rust
let p = Point { x: 1.0, y: 2.0 };
let pair = Pair(1, 2);
println!("{} {}", p.x, pair.0);   // named field vs positional (.0, .1, ...)
```

No inheritance (no `class Dog extends Animal`) — composition and traits
do what inheritance does in Java. A "tuple struct" is a lightweight way
to give a plain tuple a distinct type name (`Pair` isn't interchangeable
with `(i32, i32)` even though they look the same at runtime — the
compiler treats them as different types, which catches "passed the
coordinates in the wrong order" bugs that a bare tuple wouldn't).

## Enums — a type that's one of several *shapes*, not just values

This is the biggest structural difference from Python/Java. A Java
`enum` is a fixed set of named *constants*. A Rust `enum` is a fixed set
of named *variants*, each of which can carry its own different data:

```rust
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Point,                              // a variant can carry nothing
}
```

This is closer to Python's `Union[Circle, Rectangle, Point]` or a
tagged-union pattern — except the compiler *knows* it's one closed set
of possibilities and will force you to handle every one of them (see
`match` below). `Option<T>` (`Some(T)` / `None`) and `Result<T, E>`
(`Ok(T)` / `Err(E)`) — covered in
[Option, Result & unwrap_or_else](../foundation/option-result.md) — are
just enums defined in `std`, nothing special about them syntactically.

## `match` — exhaustive, no-fallthrough pattern matching

```rust
fn area(shape: &Shape) -> f64 {
    match shape {
        Shape::Circle { radius } => std::f64::consts::PI * radius * radius,
        Shape::Rectangle { width, height } => width * height,
        Shape::Point => 0.0,
    }
}
```

Three properties that make this stricter (and safer) than Java's
`switch`:

- **Exhaustive** — every variant must be handled, or the code doesn't
  compile. Add a new variant to `Shape` later, and every `match` on it
  across the whole codebase becomes a compile error until you handle the
  new case — the compiler finds every call site for you. `_ => ...` is
  the wildcard/"everything else" arm, same role as `default:`, but using
  it means you're opting *out* of that safety net for future variants.
- **No fallthrough** — each arm is self-contained; there's no
  Java/C-style "forgot a `break`, fell into the next case" bug class.
- **`match` is an expression** — it evaluates to a value, same as `if`
  (chapter 1). No `return` needed inside each arm.

## Destructuring — pulling values out of shapes

`match` patterns aren't limited to enum variants — they destructure
structs, tuples, and nested combinations directly:

```rust
let point = (3, 7);
match point {
    (0, 0) => println!("origin"),
    (x, 0) => println!("on the x-axis at {x}"),
    (0, y) => println!("on the y-axis at {y}"),
    (x, y) => println!("at ({x}, {y})"),
}
```

```rust
struct Point { x: i32, y: i32 }
let p = Point { x: 0, y: 7 };
let Point { x, y } = p;   // destructure directly in a `let`, no match needed
```

Richer pattern forms, all valid inside `match` (and `if let`):

```rust
match n {
    1 | 2 => "one or two",             // multiple values, one arm
    3..=9 => "three through nine",     // inclusive range pattern
    x if x < 0 => "negative",           // match guard — extra condition
    x @ 10..=20 => println!("{x}, in range"),  // @ binds the value AND matches a pattern
    _ => "something else",
}
```

## `if let` / `while let` — match when you only care about one arm

```rust
if let Shape::Circle { radius } = shape {
    println!("radius is {radius}");
}
// equivalent to a full match with `_ => {}` as the fallback arm — just terser
```

Covered in depth for the `Option`/`Result` case in
[Option, Result & unwrap_or_else](../foundation/option-result.md); same
mechanism, works on any enum.

## `#[derive(Debug)]` — printable structs/enums for free

```rust
#[derive(Debug)]
struct Point { x: i32, y: i32 }

println!("{:?}", Point { x: 1, y: 2 });   // Point { x: 1, y: 2 }
```

Unlike Python (where every object gets a default, if ugly, `repr`) and
Java (every object gets a default, if useless, `toString` — the
`Foo@1a2b3c` hash form), Rust structs/enums have **no** default way to
be printed until you ask for one — `#[derive(Debug)]` generates a
reasonable field dump; a hand-written `impl Display` gives you full
control over user-facing output. Full trait picture in
[Traits](../foundation/traits.md).
