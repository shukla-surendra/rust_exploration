# `impl` blocks: `Self`, field-init shorthand, and `&self` vs `&mut self`

> **Coming from Python/Java:** Python already makes `self` explicit in
> every method signature (`def increment(self):`) — that part of Rust
> will feel completely familiar. Java hides the equivalent (`this`) as
> implicit, so Rust's `&self`/`&mut self` will feel like Python with one
> upgrade: the *mutability* of that implicit-in-Java, explicit-in-Python
> `self` parameter is now part of the signature and checked by the
> compiler, not just a convention. The bigger structural difference from
> both: methods live in a separate `impl` block, not nested inside the
> `struct` body the way methods live inside a Python/Java `class` block.
>
> **Practical payoff:** reading a method's first parameter tells you
> everything about whether it mutates — `&self` is safe to call without
> a second thought (like calling a getter), `&mut self` means the value
> has to be a `mut` binding at the call site, and `self` (no `&`) means
> the call *consumes* the value, same category of "this uses it up" as
> Python's `list.pop()` mutating in place vs. a method that returns a
> new object instead.

A `struct` on its own is just data. Behavior gets attached separately, in
an `impl` block:

```rust
struct Counter {
    count: u32,
}

impl Counter {
    fn new() -> Self {
        Counter { count: 0 }
    }

    fn increment(&mut self) {
        self.count += 1;
    }

    fn value(&self) -> u32 {
        self.count
    }
}
```

## `Self` — "the type this block is for"

`Self` (capital `S`) is shorthand for whatever type the surrounding
`impl` block is attached to — here, `Counter`. `fn new() -> Self` means
exactly the same thing as `fn new() -> Counter`, but stays correct
automatically if the struct is later renamed or made generic (`Counter<T>`)
— you'd only have to update the `impl` line, not every method signature
inside it. `rgrep`'s `Config::build` uses the same convention:

```rust
impl Config {
    pub fn build(args: &[String]) -> Result<Config, RgrepError> {
        // could equally write `Result<Self, RgrepError>`
    }
}
```

## Field-init shorthand

```rust
fn new() -> Self {
    Counter { count: 0 }
}
```

When a local variable/parameter shares its name with a struct field,
`Field { name }` is shorthand for `Field { name: name }`. It only kicks
in when the names match exactly:

```rust
fn build(pattern: String, paths: Vec<String>) -> Config {
    Config { pattern, paths }   // both names match their fields — shorthand applies
}
```

If a field's value comes from something whose name doesn't match the
field (`Config { pattern, paths: leftover_args }`), you still write the
full `field: value` form for that one.

## `&self` vs `&mut self` — read-only vs mutating methods

The first parameter of a method (conventionally called `self`) declares
how it borrows the value it's called on, and that declaration is
enforced by the compiler, not just a naming convention:

| Signature | Means | Can it mutate `self`'s fields? |
|---|---|---|
| `fn value(&self) -> u32` | borrows immutably | no — read-only |
| `fn increment(&mut self)` | borrows mutably | yes |
| `fn into_inner(self) -> u32` | takes ownership | yes, and the caller loses access to the original value afterward |

```rust
let mut c = Counter::new();
c.increment();        // needs &mut — `c` must be declared `mut` for this to compile
println!("{}", c.value());   // read-only, `c` doesn't need to be mut for this call alone
```

This is why you'll see `&mut self` on methods that assign into `self`'s
fields (like `Config`-mutating setters, or `Vec::push`) and `&self` on
methods that only read (`Vec::len`, `Config`'s own field accessors, if
you add any) — the signature itself documents, and the compiler
enforces, whether calling a method can change the value.

## Self-referential structs: `Option<Box<Self>>`

A struct that needs to hold more instances of its own type (a tree node
holding child nodes, a linked-list node holding the next node) can't do
it with a bare field of its own type — see
[Sized vs Unsized](./sized-vs-unsized.md#scenario-3--recursive-types) for
why that fails to compile. The fix combines two things:

```rust
struct Node<T> {
    value: T,
    next: Option<Box<Node<T>>>,
}
```

- **`Box<Node<T>>`** — breaks the infinite-size problem: a `Box` is
  always a fixed-size pointer, regardless of what it points to, so
  `Node`'s size no longer depends on computing the size of another
  `Node` first.
- **`Option<...>`** — a node might be the last one, with nothing after
  it. Rust has no `null`, so "there might not be a next node" is spelled
  out as `Option`, forcing every access through `match`/`if let`/`.map()`
  to handle the "nothing there" case explicitly — see
  [Option, Result & unwrap_or_else](./option-result.md).

Reading through one of these fields from a `&mut self` method needs one
more piece of syntax worth knowing: you generally can't *move* the `Box`
out of `self.next` just to inspect or recurse into it (you only have a
borrow of `self`, not ownership), so pattern-matching against it uses
`ref`/`ref mut` to bind a reference into the existing value instead of
attempting to take it:

```rust
match self.next {
    Some(ref mut n) => n.some_method(),      // reaches into the existing Box, no move
    None => { /* base case */ }
}
```

Same idea, read-only version, written as a combinator chain instead of a
`match`:

```rust
self.next.as_ref().map(|n| n.value).unwrap_or_default()
```

`.as_ref()` turns `&Option<Box<Node<T>>>` into `Option<&Box<Node<T>>>` —
borrowing the contents rather than trying to move them — for exactly the
same reason `ref mut` was needed above, just for the read-only case.
