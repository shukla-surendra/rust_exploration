# 5. Collections, Closures & Iterators

**What this replaces:** Python's `list`/`dict`/`set` and list
comprehensions; Java's `ArrayList`/`HashMap`/`HashSet` and streams
(`.stream().map(...).collect(...)`) — Rust's iterator chains are
essentially Java streams, minus the `.stream()`/`.collect(Collectors...)`
ceremony.

## Arrays, slices, and `Vec` — three related, easily confused types

| | Fixed size, known at compile time | Size known only at runtime |
|---|---|---|
| Owns its data | `[T; N]` (array) | `Vec<T>` |
| Borrows a view into data | `&[T; N]` (rare) | `&[T]` (slice) |

```rust
let arr: [i32; 3] = [1, 2, 3];        // fixed size, on the stack
let v: Vec<i32> = vec![1, 2, 3];       // growable, heap-backed (like Python's list)
let s: &[i32] = &v[0..2];              // a slice — a borrowed view, doesn't copy
```

`Vec<T>` is what you reach for by default — it's the direct analog of
Python's `list`/Java's `ArrayList`. `&[T]` (a slice) is what function
parameters should usually accept instead of `&Vec<T>`, for the same
reason `&str` is preferred over `&String` (see
[Dereferencing](../foundation/dereferencing.md)'s deref-coercion
section) — a slice works whether the caller has a `Vec`, an array, or
another slice. `&[T]`/`[T]` are also the unsized-type case covered in
[Sized vs Unsized](../foundation/sized-vs-unsized.md).

```rust
let mut v = vec![1, 2, 3];
v.push(4);                    // append
v.pop();                      // remove & return the last element (Option<T>)
v[0];                          // index — panics if out of bounds
v.get(0);                      // Option<&T> — no panic, see Option, Result & unwrap_or_else
v.len();
for x in &v { }                // iterate by reference — v still usable after
for x in v { }                  // iterate by value — CONSUMES v (moves each element out)
```

## `HashMap` — Python's `dict`, Java's `HashMap`

```rust
use std::collections::HashMap;

let mut scores: HashMap<String, i32> = HashMap::new();
scores.insert("alice".to_string(), 10);
scores.get("alice");                        // Option<&i32> — not there? None, not an exception
*scores.entry("bob".to_string()).or_insert(0) += 1;   // "increment, or start at 0"
```

`.get()` returning `Option` instead of raising (Python's `KeyError`) or
returning `null` (Java) is the same "no null, handle absence explicitly"
theme from [Option, Result & unwrap_or_else](../foundation/option-result.md).
`.entry(key).or_insert(default)` is the idiomatic "get or insert" —
covers the common `if key not in dict: dict[key] = default` pattern in
one call. `HashSet<T>` exists too, same relationship to `HashMap` as
Python's `set` has to `dict`.

Iteration order for `HashMap`/`HashSet` is unspecified (like Python's
`dict` used to be, unlike current Python where insertion order is
guaranteed) — reach for `BTreeMap`/`BTreeSet` if you need sorted-key
iteration.

## Closures — anonymous functions that capture their environment

```rust
let factor = 3;
let multiply = |x: i32| x * factor;   // captures `factor` from the surrounding scope
println!("{}", multiply(5));           // 15
```

Directly comparable to Python lambdas (except closures can hold multiple
statements in `{ }`, not just one expression) and Java lambdas
(`x -> x * factor`, with the same "captures effectively-final variables"
flavor). You've already used one: `unwrap_or_else(|err| { ... })` in
[Error Handling in main.rs](../foundation/error-handling.md).

### The three closure traits — how a closure captures its environment

| Trait | Captures by | Can call it... |
|---|---|---|
| `Fn` | reference (`&T`) | any number of times |
| `FnMut` | mutable reference (`&mut T`) | any number of times, mutating captured state |
| `FnOnce` | by value (moves it in) | exactly once |

You rarely write these bounds yourself — they show up in function
signatures that *accept* a closure (`fn retry<F: FnMut() -> bool>(f: F)`)
and the compiler infers which one a given closure satisfies from how it
uses its captures. The intuition: a closure that only *reads* a captured
variable is `Fn`; one that mutates a captured variable is `FnMut`; one
that consumes a captured variable (e.g. calls `.into()` on it, or moves
it into a `String` it returns) is `FnOnce`.

## Iterators — the thing `for`, `.map()`, `.filter()` all run on

```rust
let v = vec![1, 2, 3, 4, 5];

let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();
let evens: Vec<&i32> = v.iter().filter(|x| **x % 2 == 0).collect();
let sum: i32 = v.iter().sum();
let total: i32 = v.iter().fold(0, |acc, x| acc + x);
```

Directly parallel to a Python list comprehension / generator expression,
or a Java `.stream().map(...).filter(...).collect(...)` chain — same
shape, terser syntax, and (unlike Python generators, closer to Java
streams) **zero-cost**: the compiler inlines and fuses the whole chain
into a single loop at compile time, no intermediate collections built
between steps.

**Lazy evaluation** — nothing runs until something *consumes* the
iterator. `v.iter().map(|x| { println!("{x}"); x * 2 })` on its own
prints nothing — the closure never actually runs until you `.collect()`,
`.sum()`, `for`-loop over it, or otherwise pull values out.

### `.iter()` vs `.into_iter()` vs `.iter_mut()`

| Method | Yields | Effect on the original collection |
|---|---|---|
| `.iter()` | `&T` (references) | untouched, still usable after |
| `.iter_mut()` | `&mut T` (mutable references) | untouched structurally, but you can mutate elements through it |
| `.into_iter()` | `T` (owned values) | **consumed** — the collection is moved from, unusable afterward |

`for x in &v` desugars to `.iter()`; `for x in v` desugars to
`.into_iter()`. This three-way split is a direct consequence of
ownership/borrowing (chapter 2) — Python/Java iteration never has to
make this choice, because references there aren't ownership-tracked the
same way.

### Common adaptors, quick reference

| Method | Does |
|---|---|
| `.map(f)` | transform each item |
| `.filter(pred)` | keep items where `pred` is true |
| `.filter_map(f)` | map + drop `None`s in one pass |
| `.fold(init, f)` | reduce to a single value, like Python's `functools.reduce` |
| `.sum()` / `.count()` / `.max()` / `.min()` | the obvious aggregates |
| `.enumerate()` | pairs each item with its index — `(0, x)`, `(1, y)`, ... |
| `.zip(other)` | pairs up two iterators elementwise |
| `.collect()` | pull everything into a concrete collection — needs a target type, often via `let x: Vec<_> = ...` or the turbofish `.collect::<Vec<_>>()` |
| `.rev()` | reverse the iteration order |
