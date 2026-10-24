# 3. `async`/`.await` in Depth

Everything here is genuinely part of the language (chapter 0) — no
runtime assumed yet.

## `async fn` vs `async` blocks

```rust
async fn fetch(url: &str) -> String {
    // ...
}

let fut = async {
    let a = fetch("a").await;
    let b = fetch("b").await;
    format!("{a}{b}")
};
```

An `async { ... }` block is an anonymous `Future`, the same relationship
a closure (`|x| x + 1`) has to a named `fn` — useful for building a
one-off future inline (e.g. to pass to `tokio::spawn`, chapter 4)
without declaring a whole named function for it.

## `.await` only works inside `async` context

```rust
fn not_async() {
    fetch("a").await;   // COMPILE ERROR: `.await` is only allowed inside `async` functions and blocks
}
```

This is the compiler enforcing chapter 1's mental model directly: `.await`
means "suspend this state machine here," which only makes sense inside
something that *is* a state machine (an `async fn`/`async` block). A
plain synchronous function has no way to suspend.

## Return types: `async fn` desugars its return type for you

```rust
async fn get_number() -> i32 { 42 }
```

is roughly equivalent, at the type level, to:

```rust
fn get_number() -> impl Future<Output = i32> {
    async { 42 }
}
```

You'll see this `impl Future<Output = T>` shape directly in generic code
that's *runtime-agnostic* (accepts any future-returning function without
committing to a specific concrete future type) — same `impl Trait`
mechanism covered generically in
[Traits](../foundation/traits.md#two-ways-to-use-a-trait-generics-vs-trait-objects),
just with `Future` as the trait.

## `Send` bounds — the error message you will eventually hit

```rust
tokio::spawn(async move {
    // ...
});
```

`tokio::spawn` (multi-threaded runtime, chapter 2's default) requires
the future you hand it to be `Send` — safe to move to another thread,
per [Concurrency](../workbook/08-concurrency.md#send-and-sync--the-traits-that-make-all-of-this-enforceable) —
because the executor's worker threads can and do move tasks between
themselves (work-stealing). This surfaces as a specific, initially
confusing error:

```
error: future cannot be sent between threads safely
  ... within this `async` block, the trait `Send` is not implemented for `Rc<...>`
```

The fix is almost always the fix you'd expect from
[Memory & Smart Pointers](../workbook/07-memory-and-smart-pointers.md):
swap a non-`Send` type (`Rc<T>`, a `RefCell<T>` guard held across an
`.await`) for its thread-safe counterpart (`Arc<T>`, or restructure so
the non-`Send` value doesn't need to live across a suspend point).
`#[tokio::main(flavor = "current_thread")]` (chapter 2) sidesteps this
requirement entirely — single-threaded execution has no work-stealing,
so `Send` isn't needed — a legitimate escape hatch for a small,
genuinely single-threaded CLI tool.

## Lifetimes in `async fn` — one elision rule that surprises people

```rust
async fn print_it(s: &str) {
    // some .await in here
    println!("{s}");
}
```

desugars to a future whose type **borrows `s` for its own lifetime** —
i.e. the generated state machine holds a reference into whatever called
it, so the caller's value must stay alive for as long as the returned
future is being polled, not just for the duration of the call. This
occasionally produces lifetime errors that look surprising if you're
thinking of `async fn` as "just a function" rather than "a function that
returns a value borrowing from its arguments" — the fix, same toolkit as
[Ownership, Borrowing & Lifetimes](../workbook/02-ownership-borrowing-lifetimes.md),
is usually to take an owned value (`String` instead of `&str`) if the
future needs to outlive the caller's borrow (e.g. because it's being
`tokio::spawn`-ed, which requires `'static` — no borrowed data at all).

## `dyn Future` and boxing — when you need a future behind a pointer

Same story as [Sized vs Unsized](../foundation/sized-vs-unsized.md) for
`dyn Trait` generally: different concrete `async fn`s produce different,
differently-sized, compiler-generated state machine types, so you can't
put "some future" directly in a `Vec` or a struct field without
indirection:

```rust
let futures: Vec<std::pin::Pin<Box<dyn Future<Output = i32>>>> = vec![
    Box::pin(fetch_a()),
    Box::pin(fetch_b()),
];
```

`Box::pin(...)` does two things at once: boxes the future (heap
allocation, fixed-size pointer — chapter 7's `Box<T>` from the workbook)
and pins it (chapter 1's `Pin`, since it now needs a stable address).
You'll see this exact shape again in
[Concurrency Patterns](./04-concurrency-patterns.md)'s `FuturesUnordered`
section — it's the standard way to hold a heterogeneous collection of
futures.
