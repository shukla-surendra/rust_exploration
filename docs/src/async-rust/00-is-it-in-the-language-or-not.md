# 0. Is async actually "in" Rust, or bolted on?

You asked this directly, and the honest answer is: **both, split down a
specific line** — and that line is the single most important thing to
get straight before anything else in this tutorial makes sense.

## The split, precisely

| Piece | Where it lives | Who defines it |
|---|---|---|
| `async`/`.await` **syntax** | the language itself | the Rust compiler |
| The `Future` trait | `core`/`std` (`std::future::Future`) | the standard library |
| `Pin`, `Waker`, `Context` (the machinery `Future` is built on) | `core`/`std` | the standard library |
| **The executor** (the thing that actually runs a `Future` to completion) | **not in `std` at all** | an external crate — `tokio`, `async-std`, `smol`, ... |
| Async I/O (non-blocking file/socket reads, timers) | **not in `std`** | the runtime crate you pick |

So: the *grammar* of async Rust — writing `async fn`, writing `.await`,
the `Future` trait your `async fn` compiles down to — is genuinely part
of the language, checked by the compiler, no crate required. What's
missing from `std` is the thing that actually **drives** a `Future`
forward: no executor, no event loop, no non-blocking I/O. That's why
every single async example in this tutorial (and in
[Rust in the AWS Ecosystem](../production/11-aws-ecosystem.md)) starts
with `#[tokio::main]` or an explicit `tokio::runtime::Runtime` — without
picking a runtime crate, `async fn main() { ... }` compiles, but nothing
ever actually calls it.

## Proving it to yourself

```rust
async fn hello() {
    println!("hello");
}

fn main() {
    hello();   // compiles! prints NOTHING.
}
```

This compiles cleanly and prints nothing at all — a real, easy-to-hit
bug. `hello()` doesn't run the function body; it constructs a `Future`
value (a suspended computation) and immediately drops it, unpolled. The
`println!` inside never executes. Compare:

```rust
#[tokio::main]
async fn main() {
    hello().await;   // now it actually runs
}
```

`.await` is what asks *something* to poll the `Future` until it's done.
`#[tokio::main]` is what supplies that "something" (an executor) for
your top-level `main`. Nothing here is `std` magic — `#[tokio::main]` is
a macro from the `tokio` crate that expands roughly to "build a `tokio`
runtime, hand it your async body, block until it finishes."

## Why Rust designed it this way (and why Python didn't)

**Python's `asyncio` is one specific, batteries-included choice**,
shipped in the standard library — there's exactly one event loop
implementation you're expected to use (barring `uvloop` as a drop-in
replacement), and `import asyncio` just works.

Rust deliberately didn't do that, for a reason that matters once you've
used the language a while: Rust runs in wildly different environments
that need wildly different executors — a web server wants a
multi-threaded, work-stealing scheduler (`tokio`'s default); an
embedded microcontroller with no OS wants a `no_std` executor with zero
heap allocation (`embassy`); a single-threaded WASM target in a browser
needs yet another shape entirely. Baking one executor into `std` would
force every one of those to carry machinery they don't need, or force
`std` itself to somehow support all of them — genuinely not a problem
Python's typical deployment targets (a server, a script) have to solve.
So the standard library ships the *contract* (`Future`, `.await`) and
lets the ecosystem compete on *implementations* of that contract.

## What this means practically, day to day

- **You must pick a runtime before writing meaningful async code.**
  [Choosing a Runtime](./02-choosing-a-runtime.md) covers the real
  options — `tokio` is the correct default choice for almost everything
  you'll do, including every AWS SDK example already in this repo.
- **Two crates built on *different* runtimes don't automatically mix.**
  This is the sharpest edge of Rust's "no batteries in std" design, and
  the topic of [Choosing a Runtime](./02-choosing-a-runtime.md)'s
  "runtime lock-in" section — nothing like this exists in Python's
  world, where `asyncio` is simply the only real option.
- **The language-level pieces (`async fn`, `.await`, `Future`) transfer
  everywhere**, regardless of which runtime you eventually pick — that
  part really is "just Rust," the same way `if`/`match` are, and that's
  what [`async`/`.await` in Depth](./03-async-fn-await-in-depth.md)
  covers before ever assuming `tokio` specifically.
