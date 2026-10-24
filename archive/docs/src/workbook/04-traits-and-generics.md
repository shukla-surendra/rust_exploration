# 4. Traits & Generics

**What this replaces:** Java interfaces (traits) and Java/C++ generics
(`<T>` — the syntax is nearly identical). Python doesn't really have
either at the language level (duck typing covers the trait use case;
Python generics/`TypeVar` are optional, checked by an external tool
`mypy`, not the runtime). This chapter is a condensed pass — full depth,
including static vs dynamic dispatch and the standard-library trait
table, is in [Traits](../foundation/traits.md).

## Traits, in one line

A trait is a checklist of methods a type promises to implement — Rust's
answer to Java's `interface`, but usable on types you don't own (see
"Extending types you don't own" in the full page) and consumable either
at compile time (zero cost) or runtime (via a pointer). If you only read
one linked page from this whole workbook, make it
[Traits](../foundation/traits.md) — ownership/borrowing and traits are
the two concepts that actually differ from what you already know.

```rust
trait Shape {
    fn area(&self) -> f64;
    fn describe(&self) -> String {          // default method — optional to override
        format!("area = {:.2}", self.area())
    }
}

impl Shape for Circle {
    fn area(&self) -> f64 { /* ... */ }
}
```

## Generics — parameterized over types

```rust
fn largest<T: PartialOrd>(items: &[T]) -> &T {
    let mut max = &items[0];
    for item in items {
        if item > max { max = item; }
    }
    max
}
```

`<T: PartialOrd>` reads "for any type `T` that supports ordering
comparisons." Compare to Java's `<T extends Comparable<T>>` — same idea,
different spelling: a **bound** restricts which types can be plugged in,
based on required *behavior* (a trait), not a base class. Unbounded
generics (`<T>` with no `:` bound) mean "works for absolutely any type,"
useful when the function never actually operates on `T`'s contents (e.g.
a generic container).

Generic structs and enums work the same way — you've already been using
two of the most common ones:

```rust
struct Wrapper<T> { value: T }

enum Option<T> { Some(T), None }        // yes, this is how Option is actually defined
enum Result<T, E> { Ok(T), Err(E) }     // and this is Result
```

## `where` clauses — bounds, but readable

```rust
fn process<T: Clone + std::fmt::Debug, U: Default>(a: T, b: U) { /* ... */ }

// same thing, easier to read once there are several bounds:
fn process<T, U>(a: T, b: U)
where
    T: Clone + std::fmt::Debug,
    U: Default,
{ /* ... */ }
```

Purely a readability tool — `where` doesn't change what compiles, just
how it's laid out once bounds get long.

## Monomorphization — why generics are "free" at runtime

Unlike Java generics (type-erased — `List<String>` and `List<Integer>`
share one compiled class at runtime, with casts inserted) or Python
(no compile step at all), Rust generates a **separate compiled copy** of
a generic function for every concrete type it's actually called with.
`largest::<i32>` and `largest::<f64>` are two distinct functions in the
final binary, each fully specialized — no runtime type checks, no
boxing, direct calls the same as if you'd hand-written both versions.
This is also why generic code can be slow to compile (more code
generated) but fast to run (nothing generic left by the time it runs) —
see the static-dispatch discussion in
[Traits](../foundation/traits.md#two-ways-to-use-a-trait-generics-vs-trait-objects)
for the `dyn Trait` alternative when you need the opposite tradeoff.

## Associated types — a brief preview

You'll see this shape in `Iterator` (chapter 5) and won't need to write
it yourself often, but it's worth recognizing:

```rust
trait Container {
    type Item;                       // a type that's part of the trait's "interface"
    fn get(&self, i: usize) -> Self::Item;
}
```

Different from a generic parameter (`Container<T>`) in that each
implementor picks *one* concrete `Item` type, not "however many types
you want to instantiate it with" — used when a trait's other methods
only make sense for a single, implementor-chosen type.
