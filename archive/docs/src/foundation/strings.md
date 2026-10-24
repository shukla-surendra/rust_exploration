# `String` vs `str` / `&str`

> **Coming from Python/Java:** Python's `str` and Java's `String` are
> each a single type — you never choose between an "owned" and
> "borrowed" version, because both languages quietly garbage-collect
> whichever one you're not using anymore. Rust makes that choice
> explicit: `String` is the type you already know (owned, growable,
> mutable), and `&str` is a *view* into string data someone else owns —
> closer to a Python slice (`s[1:4]`, which also doesn't copy) than to
> anything Java's `String` distinguishes.
>
> **Practical rule of thumb**, and the one that matters day to day:
> accept `&str` in function parameters (like a Python function just
> takes a string, no ownership question asked), and return/store
> `String` only when you're building new data that needs to outlive the
> function. Getting this backwards (taking `String` everywhere) is the
> single most common "why do I need all these `.clone()`/`.to_string()`
> calls" experience for people coming from Python or Java.

## `str` — the unsized string type

`str` is a *dynamically sized type* (DST) — the compiler doesn't know its
size at compile time, since a string can be any length. Rust can't put a
DST directly in a variable, on the stack, or pass it by value — it has no
fixed size to allocate. That's why you almost never see bare `str`; you
always see it behind a pointer:

- `&str` — a borrowed, immutable view into string data (a "string slice")
- `Box<str>` — an owned, heap-allocated `str` with no extra capacity
- `Rc<str>` / `Arc<str>` — shared-ownership variants

## `&str` — string slice

`&str` is what you'll use almost everywhere. It's a *fat pointer*: a
pointer to some UTF-8 bytes plus a length. It never owns the data — it
just borrows it. The data it points to can live:

- **in the binary itself** — string literals, e.g. `"hello"`, have type
  `&'static str`. They're baked into the compiled binary and live for the
  entire program (`'static` lifetime).
- **inside a `String`** — `&my_string` (or `my_string.as_str()`) borrows a
  slice of a heap-allocated `String`.
- **inside another `&str`** — slicing, e.g. `&s[0..3]`.

```rust
fn greeting() -> &'static str {
    "Hello, welcome to our application!"   // literal, baked into the binary
}
```

This is exactly what `src/welcome.rs` does — the literal has `'static`
lifetime, so returning `&'static str` is valid: the data isn't owned by
any stack frame that could go out of scope.

## `String` — owned, growable string

`String` is a heap-allocated, owned, growable buffer of UTF-8 bytes
(essentially `Vec<u8>` with UTF-8 guarantees). Because it owns its data,
it can be mutated (`push_str`, `push`, `+=`) and outlives the function
that created it without any lifetime annotation.

```rust
let s: String = String::from("hello");
let slice: &str = &s;          // borrow it as a &str
let owned: String = slice.to_string();  // or slice.to_owned() — clones into a new String
```

## Choosing between them

| | `&str` | `String` |
|---|---|---|
| Ownership | borrowed | owned |
| Mutable | no | yes |
| Allocation | none (points at existing data) | heap-allocated |
| Typical use | function parameters, literals, read-only views | struct fields you need to own/mutate, building strings at runtime |

Rule of thumb: **accept `&str` in function parameters** (it accepts both
`&String` and `&str` via deref coercion, so it's more flexible for
callers), and **return/store `String` when you need ownership** — e.g. the
value is built at runtime and must outlive the function, or a struct needs
to own its data instead of borrowing it.

```rust
fn shout(s: &str) -> String {      // borrow in, own out
    format!("{}!", s.to_uppercase())
}
```
