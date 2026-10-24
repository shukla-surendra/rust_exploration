# Dereferencing in Rust

> **Coming from C, more than Python/Java:** this is the one page in this
> repo where your C background is the useful reference, not
> Python/Java — neither of those languages exposes pointers or a `*`
> operator at all (a Java/Python reference is followed *for* you,
> invisibly, every time you use it). Rust's `*` is mechanically the same
> idea as C's `*ptr` — "follow this pointer to the value" — just with two
> upgrades: the compiler tracks *what kind* of pointer you have (`&T`
> vs `Box<T>` vs...) and picks the right mechanism automatically via the
> `Deref` trait, and it inserts `*` for you at most call sites (deref
> coercion, method lookup) so you rarely have to write it by hand the
> way C requires everywhere.
>
> **Practical payoff:** if `*first + *second` in
> [`heap_allocation.rs`](../../../variables/src/bin/heap_allocation.rs)
> read immediately as "dereference two pointers and add what they point
> to," you already have the right mental model from C — this page is
> mostly about *when Rust does that step for you automatically* so you
> stop reaching for `*` out of habit where it isn't needed.

Dereferencing means following a pointer to get at the value it points
to. Rust has several kinds of pointer-like values, and the `*` operator
means "give me the thing this points to" for all of them — but *how*
that happens, and how often you actually have to type `*` yourself,
differs by type.

## The basic case: references

A reference (`&T`, `&mut T`) is a borrowed pointer to a value owned by
someone else. `*` on a reference is a direct, built-in operation — no
trait involved:

```rust
let x = 5;
let r = &x;
assert_eq!(*r, 5);       // follow the reference to read the value

let mut y = 10;
let m = &mut y;
*m += 1;                  // follow the reference to mutate the value
assert_eq!(y, 11);
```

`r` itself is a pointer (an address); `*r` is the `i32` at that address.

## Smart pointers: `Deref` and `DerefMut`

Types like `Box<T>`, `Rc<T>`, and `RefCell<T>`'s borrow guards aren't
references, but they *behave* like one because they implement the
`Deref` trait from `std::ops`:

```rust
use std::ops::Deref;

struct MyBox<T>(T);

impl<T> Deref for MyBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}
```

This is exactly what makes `*first + *second` work in
[`heap_allocation.rs`](../../../variables/src/bin/heap_allocation.rs):

```rust
let first: Box<i32> = Box::new(5);
let sum = *first + 1;   // *first calls Deref::deref(&first), yielding &i32,
                         // then that's auto-dereferenced again to i32
```

`*first` desugars to `*(first.deref())` — `deref()` returns `&i32`, and
the outer `*` then reads through that reference. You don't write `deref()`
yourself; the `*` operator finds and calls it for you whenever the type
implements `Deref`.

`DerefMut` is the mutable counterpart — implement it and `*my_box = 6`
or `*my_box += 1` work too, for types that need to hand back a mutable
reference to their inner data.

## Deref coercion — why you rarely need `*` at all

Rust automatically inserts `Deref` calls at function-call and
method-call boundaries so a `&Box<T>` (or `&String`, `&Rc<T>`, ...) can
be used wherever a `&T` is expected, without you writing `*` manually:

```rust
fn greet(name: &str) {
    println!("hello, {name}");
}

let owned = String::from("Ferris");
greet(&owned);   // &String coerces to &str — no need for &*owned
```

The chain here is `&String → &str` via `String`'s `Deref<Target = str>`
impl. Coercion can chain through multiple layers too — `&Box<String>`
would coerce all the way to `&str` if needed. This is also why you can
call `String` methods like `.len()` or `.push_str()` directly: method
lookup follows the `Deref` chain automatically (see the next section).

## Method calls: auto-ref and auto-deref

When you write `value.method()`, Rust doesn't require `value`'s type to
match the method's `self` type exactly. It automatically inserts `&`,
`&mut`, or `*` — as many times as needed, in any combination — to find
an impl that fits. This is why both of these work without you ever
writing `*`:

```rust
let boxed: Box<Vec<i32>> = Box::new(vec![1, 2, 3]);
boxed.push(4);       // Vec::push takes &mut self — auto-deref: (*boxed).push(4)
let n = boxed.len();  // Vec::len takes &self  — auto-deref: (*boxed).len()
```

## Comparing dereferencing to indexing sugar

`[]` indexing is a related but separate mechanism (`Index`/`IndexMut`
traits, not `Deref`), but it composes with deref coercion in the same
spirit — `Vec<T>` doesn't implement `Index` itself for slicing, it
derefs to `[T]` which does. The underlying idea is the same throughout
Rust: **types that "stand in" for another type implement `Deref`, and
the compiler inserts `*` for you at the points where it's unambiguous
(method calls, coercion sites) — you only write `*` explicitly when
using a value directly in an expression, as `*first + *second` does.**

## Summary

| You have | `*` does |
|---|---|
| `&T` / `&mut T` | built-in: read/write the pointee directly |
| `Box<T>`, `Rc<T>`, `MutexGuard<T>`, ... | calls `Deref::deref()` (or `DerefMut::deref_mut()`), then reads/writes through the returned reference |
| function/method argument of `&Box<T>` where `&T` expected | inserted automatically by *deref coercion* — you never write `*` |
| `receiver.method()` where types don't match `self` exactly | compiler inserts `&`/`&mut`/`*` automatically via *method lookup* — you never write `*` |

Explicit `*` is mainly needed when you use the pointed-to value directly
in an expression (arithmetic, comparisons, passing by value) rather than
calling a method or passing a reference — which is exactly the case in
`*first + *second`.
