# When doesn't Rust know a value's size at compile time?

[Stack vs Heap](./stack-vs-heap.md) claimed Rust defaults to the stack
because "sizes are known at compile time." This page answers the
follow-up: when does that break down, and does strict typing cause it or
prevent it?

## Two separate questions

It's easy to conflate these, but Rust keeps them distinct:

1. **"What type is this value?"** — always resolved at compile time in
   Rust. This is what strict/static typing means: no value's type is a
   mystery, or changes, at runtime.
2. **"How many bytes does this value occupy?"** — usually also known at
   compile time (that's the `Sized` trait, which every generic type
   parameter implicitly requires unless you opt out with `?Sized`). But
   *knowing the type* doesn't automatically imply *knowing the byte size*
   — a handful of types are fully known and fully type-checked, yet have
   no fixed size.

So the honest framing is: **strict typing is not what causes the
"unknown size" cases below — it's what lets the compiler catch them.**
A weakly/dynamically typed language wouldn't even have a compile-time
concept of "this type's size is unknown"; it would just find out at
runtime, possibly by corrupting memory. Rust's type checker enforces the
`Sized` bound precisely so this can never happen silently — it forces
you to add a pointer the moment size-unknown-ness shows up.

## Scenario 1 — Dynamically Sized Types (DSTs): `str`, `[T]`, `dyn Trait`

These types are fully type-checked, but two different *values* of the
same type can have different sizes, and the type definition gives no
upper bound:

```rust
let a: str = *"hi";      // ERROR: `str` is unsized
let b: str = *"hello!";  // a different length, same type `str`
```

`"hi"` and `"hello!"` are both valid `str` data, but one is 2 bytes and
the other is 6. There's no single stack-slot size that fits "a `str`" in
general — the size is a property of the specific value, known only once
you have actual data in hand (at runtime, or from a specific literal).
The fix is always indirection: put a *pointer* on the stack instead of
the data itself. The pointer's size is fixed even though what it points
to isn't:

```rust
let a: &str = "hi";        // fat pointer: data ptr + length (16 bytes on 64-bit)
let b: Box<str> = "hi".into();  // owned, heap-allocated, same fat-pointer shape
```

`[T]` (an unsized slice, as opposed to `[T; N]`, a sized array) is the
same story: length isn't part of the type, so it isn't part of the
compile-time size. You always meet it as `&[T]` or `Box<[T]>`.

## Scenario 2 — Trait objects: `dyn Trait`

```rust
trait Draw { fn draw(&self); }
struct Circle; struct Square;
impl Draw for Circle { fn draw(&self) {} }
impl Draw for Square { fn draw(&self) {} }
```

`dyn Draw` is a fully checked type — the compiler verifies every method
call against the trait — but "something that implements `Draw`" could be
a `Circle` (0 bytes), a `Square` (0 bytes), or some other struct with
many fields (arbitrarily many bytes). The trait says nothing about
layout, only about behavior, so there's no fixed size to reserve on the
stack:

```rust
let shapes: Vec<Box<dyn Draw>> = vec![Box::new(Circle), Box::new(Square)];
```

Each `Box<dyn Draw>` is a fixed-size fat pointer (data pointer + vtable
pointer) regardless of which concrete type is boxed — that's what makes
storing different concrete types in the same `Vec` possible at all.

## Scenario 3 — Recursive types

```rust
enum List {
    Cons(i32, List),   // ERROR: recursive type has infinite size
    Nil,
}
```

Here every part of the type *is* named and checked — there's no
ambiguity about what `List` is. The problem is arithmetic: computing
`size_of::<List>()` requires first computing `size_of::<List>()`, which
never bottoms out. Wrapping the recursive field in `Box` fixes it not by
changing the type checking, but by breaking the size computation's
infinite regress — `Box<List>` is always pointer-sized, no matter how
big `List` eventually turns out to be:

```rust
enum List {
    Cons(i32, Box<List>),  // OK — size = i32 + one pointer, always
    Nil,
}
```

## Scenario 4 — Collections whose *contents* are runtime-determined

This one is different from the first three: `Vec<T>`, `String`, and
`HashMap<K, V>` are all `Sized` — the compiler knows exactly how big the
`Vec<T>` struct itself is (a pointer, a length, and a capacity — 24
bytes on a 64-bit target, always, regardless of `T`). What's *not* known
at compile time is how many elements will end up in it, because that
number depends on things that only exist while the program is running:

```rust
let mut names: Vec<String> = Vec::new();
for line in std::io::stdin().lines() {   // number of lines: unknowable until run
    names.push(line.unwrap());
}
```

User input, file contents, network responses, a loop bound computed
from another runtime value — none of these are compile-time constants,
so the memory to hold them can't be reserved on the stack. `Vec`/`String`
solve this by keeping a small, fixed-size handle on the stack (pointer +
len + capacity) that owns a heap buffer it can grow or shrink on demand.
This is exactly the "opt into the heap on purpose" move described in
[Stack vs Heap](./stack-vs-heap.md).

## Summary

| Scenario | Type known at compile time? | Size known at compile time? | Fix |
|---|---|---|---|
| `str`, `[T]` | yes | no — varies per value | `&str`, `Box<str>`, `&[T]`, `Box<[T]>` |
| `dyn Trait` | yes (as an interface) | no — varies per concrete type | `&dyn Trait`, `Box<dyn Trait>` |
| Recursive `enum`/`struct` | yes | no — self-referential, diverges | `Box<Self>` inside the recursive variant |
| `Vec<T>` / `String` / `HashMap` | yes, and size of the *handle* is fixed | the handle's size is known; the *data volume* isn't (runtime-determined) | heap buffer behind a fixed-size handle (built in) |

The throughline: Rust's strict typing never fails to know *what* a value
is. What it sometimes can't know is *how many bytes* that value needs —
either because the type permits variable-length values (`str`, `[T]`,
`dyn Trait`), because the type's size calculation is self-referential
(recursive types), or because the amount of data is a genuinely runtime
quantity (user/file/network input). In every case the fix is the same
shape: put a fixed-size pointer on the stack and let it own/borrow
variable-size data on the heap.
