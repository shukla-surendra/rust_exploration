# Traits: from layman's terms to first principles

> **Coming from Python/Java:** Java's `interface` is the closest direct
> match — a trait is a checklist of methods, implemented separately from
> the data, exactly like `implements Drawable`. Python has no compiler-
> enforced equivalent by default (duck typing means "just have the
> method and hope for the best," and `typing.Protocol`/`abc.ABC` are
> optional, checked by an external tool at best) — Rust's traits give
> you Java's compile-time guarantee, but, unlike Java, you can implement
> a trait for a type you don't own (see "Extending types you don't own"
> below) — closer to Python's willingness to monkey-patch, but checked
> and safe.
>
> **Practical payoff:** almost every "how do I do X generically" question
> in Rust reduces to "which trait provides X" — printing (`Display`),
> comparing (`PartialOrd`), copying (`Clone`), converting (`From`),
> iterating (`Iterator`). Learning to recognize a trait bound in a
> function signature (`fn f<T: Display>(x: T)`) is learning to read "what
> this function actually needs from its argument," the same way a Java
> `interface` parameter tells you the same thing.

## Layman's version

A trait is a **promise a type makes about what it can do**, written down
so the compiler can check it. If a type implements the `Draw` trait,
you're promising "you can call `.draw()` on me." That's it — a trait
doesn't hold data or exist as a value by itself; it's a checklist of
behavior that concrete types opt into.

The everyday analogy: a job posting lists required skills ("must be able
to drive, must be able to lift 50lbs"), not which specific person will
show up. A trait lists required *methods*, not which specific type will
provide them. Anyone/anything meeting the checklist qualifies.

If you've used other languages, the nearest familiar concept is an
**interface** (Java/C#/TypeScript) or a **protocol** (Swift). Rust's
traits are that idea, generalized further — they can also add methods to
existing types you don't own (see "Extending types you don't own" below),
which interfaces typically can't do.

## First principles: what problem traits solve

Without traits, a function can only work with one exact type:

```rust
fn print_area_circle(c: &Circle) { println!("{}", c.area()); }
fn print_area_square(s: &Square) { println!("{}", s.area()); }
```

Every new shape needs a new function, even though the *logic* — "call
`.area()`, print it" — never changes. What varies is the type; what
should stay fixed is the code. Traits let you write the function once,
against the *behavior* you need, and let any qualifying type plug in:

```rust
trait Shape {
    fn area(&self) -> f64;
}

fn print_area(s: &impl Shape) {
    println!("{}", s.area());
}
```

This is the core purpose of a trait: **decouple "what code needs" from
"which concrete type provides it."** Everything else in this doc is a
consequence of that one idea.

## Defining and implementing a trait

```rust
trait Shape {
    fn area(&self) -> f64;              // required — no body, every impl must provide one
    fn describe(&self) -> String {       // default — has a body, impls may override it
        format!("a shape with area {:.2}", self.area())
    }
}

struct Circle { radius: f64 }
struct Square { side: f64 }

impl Shape for Circle {
    fn area(&self) -> f64 { std::f64::consts::PI * self.radius * self.radius }
}

impl Shape for Square {
    fn area(&self) -> f64 { self.side * self.side }
}
```

`Circle` and `Square` share no inheritance relationship and know nothing
about each other. `Shape` is the only thing connecting them, and it's
purely a behavioral contract — no shared fields, no shared base type.
This is deliberate: Rust has no class inheritance, so "shared behavior"
always means "shared trait," never "shared parent class."

## Two ways to use a trait: generics vs trait objects

This is where traits connect back to [Sized vs Unsized](./sized-vs-unsized.md)
and [Stack vs Heap](./stack-vs-heap.md) — the choice here is a choice
about *when* the concrete type gets decided.

### Static dispatch — `impl Trait` / generics

```rust
fn print_area<T: Shape>(s: &T) {
    println!("{}", s.area());
}
```

The compiler generates a separate copy of `print_area` for every
concrete type it's called with (`print_area::<Circle>`,
`print_area::<Square>`, ...) — this is called *monomorphization*. Each
copy calls `area()` as a direct, known function call. No indirection, no
runtime cost, but the concrete type must be known at compile time at
every call site.

### Dynamic dispatch — `dyn Trait`

```rust
let shapes: Vec<Box<dyn Shape>> = vec![
    Box::new(Circle { radius: 1.0 }),
    Box::new(Square { side: 2.0 }),
];
for s in &shapes {
    println!("{}", s.area());   // which area() runs is decided at runtime
}
```

Here the concrete type is *not* known at compile time — the whole point
is mixing `Circle` and `Square` in one `Vec`. `dyn Shape` is exactly the
unsized type described in
[Sized vs Unsized](./sized-vs-unsized.md#scenario-2--trait-objects-dyn-trait):
its size varies by concrete type, so it must live behind a pointer
(`Box<dyn Shape>` or `&dyn Shape`), and calling `.area()` goes through a
vtable lookup at runtime instead of a direct call.

| | Static dispatch (`impl Trait` / `<T: Trait>`) | Dynamic dispatch (`dyn Trait`) |
|---|---|---|
| Concrete type decided | compile time | runtime |
| Call cost | direct call, can be inlined | one vtable indirection |
| Binary size | one copy per concrete type used | one shared copy |
| Can mix types in one collection | no | yes |

## Extending types you don't own

You can implement a trait you wrote for a type from another crate, or a
trait from another crate for a type you wrote — as long as *you* own
either the trait or the type (Rust's "orphan rule," which prevents two
crates from conflicting over the same impl):

```rust
trait Loud {
    fn shout(&self) -> String;
}

impl Loud for String {          // your trait, someone else's type (std)
    fn shout(&self) -> String {
        self.to_uppercase() + "!"
    }
}

let s = String::from("hello");
println!("{}", s.shout());      // "HELLO!"
```

This is something class-based inheritance generally can't do — you'd
need to own `String`'s source to add a method to it. Traits decouple
"who defines the behavior" from "who defines the type," so either side
can be external.

## Predefined traits you already use constantly

You've been using traits from `std` throughout this project without
necessarily naming them. This is the payoff of the whole system: the
standard library expresses nearly every generic capability as a trait,
so learning "what trait does X" tells you exactly what a type supports.

| Trait | What it means | Where you've seen it |
|---|---|---|
| `Deref` / `DerefMut` | "I can be treated like a pointer to `T`" | `*first` in [`heap_allocation.rs`](../../../variables/src/bin/heap_allocation.rs), covered fully in [Dereferencing](./dereferencing.md) |
| `Drop` | "run this code automatically when I go out of scope" | why `Box`'s heap memory frees itself — see [Stack vs Heap](./stack-vs-heap.md) |
| `Clone` | "I can make an explicit, possibly-expensive duplicate of myself" (`.clone()`) | `slice.to_string()` / `.to_owned()` in [String vs str / &str](./strings.md) go through related conversion traits |
| `Copy` | "I'm cheap enough that assignment/passing implicitly duplicates me instead of moving me" (stack-only data, no heap pointers) | why `*first + *second` can read `first` and `second` without moving them — `i32` is `Copy` |
| `Debug` | "I can be printed with `{:?}`" for developers | `println!("{:?}", ...)` |
| `Display` | "I can be printed with `{}`" for end users | `println!("{}", sum)` in `heap_allocation.rs` |
| `PartialEq` / `Eq` | "I support `==` / `!=`" | `assert_eq!` in code examples |
| `PartialOrd` / `Ord` | "I support `<`, `>`, sorting" | `.sort()` on a `Vec` |
| `Default` | "I have a sensible zero-argument default value" | `Vec::new()`-style APIs, `#[derive(Default)]` |
| `Iterator` | "I can produce a sequence of values, one `.next()` at a time" | `for` loops, `.map()`, `.filter()` |
| `From` / `Into` | "I can be converted from/to another type" | `String::from("hello")`, `.into()` |
| `Sized` | "my size is known at compile time" (implicit on every type parameter unless relaxed with `?Sized`) | the entire subject of [Sized vs Unsized](./sized-vs-unsized.md) |

Most of these can be auto-implemented with `#[derive(...)]` instead of
hand-writing the impl, when your type's fields already satisfy the
trait:

```rust
#[derive(Debug, Clone, PartialEq, Default)]
struct Point { x: i32, y: i32 }
```

`derive` generates the impl by delegating field-by-field — `Point`
derives `Clone` because both `i32` fields are `Clone`, `PartialEq`
because both fields are `PartialEq`, and so on.

## Trait bounds — requiring a capability, not a specific type

```rust
fn largest<T: PartialOrd + Copy>(items: &[T]) -> T {
    let mut max = items[0];
    for &item in items {
        if item > max { max = item; }
    }
    max
}
```

`T: PartialOrd + Copy` reads as "any `T`, as long as it supports
ordering comparisons and cheap copying" — not "any `T`, as long as it's
`i32`." This is the traits system doing exactly what the first-principles
section described: the function is written once, against required
behavior, and works for `i32`, `f64`, `char`, or any future type that
happens to satisfy both bounds.

## `Ord` vs `PartialOrd` — when "comparable" isn't guaranteed

The example above bounded `T` with `PartialOrd`, not `Ord` — that choice
isn't arbitrary, and picking the wrong one is a common source of
"why won't this compile" confusion. Consider a generic struct that needs
to compare values internally:

```rust
struct Sorted<T: Ord> {
    items: Vec<T>,
}
```

`T: Ord` means "this type has a *total* ordering" — every pair of values
is comparable, and one is definitively less than, equal to, or greater
than the other. `Ord` is actually three smaller traits bundled together:

- **`PartialEq`** — defines `==` and `!=`.
- **`PartialOrd`** — defines `<`, `>`, `<=`, `>=`, but allows pairs that
  are *incomparable*: its comparison method returns `Option<Ordering>`,
  where `None` means "can't say."
- **`Eq`** — a marker trait (no new methods) on top of `PartialEq`,
  promising equality is *reflexive* (`x == x` always holds).

`Ord` requires both `Eq` and `PartialOrd`, and adds
`cmp(&self, other: &Self) -> Ordering` — returning `Less`/`Equal`/`Greater`,
never wrapped in `Option`, so there's always a definite answer.

**Why this distinction exists, concretely:** `f64`/`f32` implement
`PartialOrd` but deliberately *do not* implement `Ord` or `Eq`. The
reason is `NaN` ("not a number," the result of things like `0.0 / 0.0`):
under IEEE 754, `NaN < 1.0`, `NaN > 1.0`, and even `NaN == NaN` are all
`false` — `NaN` is incomparable to everything, including itself, which
violates both `Eq`'s reflexivity promise and `Ord`'s "always a definite
answer" guarantee. So `Sorted<f64>` simply fails to compile — the
compiler catches "this type can't guarantee a total order" at the type
level, before you ever ship code that would silently mis-sort or
infinite-loop on a stray `NaN`. Using `PartialOrd` as a bound instead
(as the `largest` example above does) is how you write code that's
allowed to work with `f64` — at the cost of the caller needing to have
ruled out `NaN` some other way, since the bound no longer guarantees it
for you.

Types that already implement `Ord` and can be used as-is: `i32`, `u32`,
`char`, `String`, `bool`, and any `enum`/`struct` carrying
`#[derive(Ord, PartialOrd, Eq, PartialEq)]` — all four are conventionally
derived together, since `Ord` needs the other three to exist first.

## Summary

- **Layman:** a trait is a checklist of behavior a type promises to
  provide.
- **First principles:** traits exist to decouple *what code needs* from
  *which concrete type supplies it*, without inheritance.
- **Mechanically:** a trait declares required (and optionally default)
  methods; types opt in via `impl Trait for Type`; you consume a trait
  either statically (`impl Trait` / generics — compiled per type, zero
  runtime cost) or dynamically (`dyn Trait` — one shared implementation,
  vtable dispatch, requires a pointer since `dyn Trait` is unsized).
- **In practice:** almost every piece of "generic-feeling" behavior in
  Rust — printing, comparing, converting, iterating, dereferencing,
  cleaning up on drop — is a predefined trait from `std`, often
  available for free via `#[derive(...)]`.
