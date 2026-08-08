# 2. Choosing a Runtime

**Python contrast:** there's no equivalent chapter needed for Python —
`asyncio` is the standard library's event loop, full stop (`uvloop`
exists as a drop-in accelerant, but it implements `asyncio`'s own
interfaces, not a competing API). Rust genuinely has multiple,
incompatible executors, because — as
[Chapter 0](./00-is-it-in-the-language-or-not.md) covered — none of them
ship in `std`. Picking one is a real, first-class decision.

## The options, and when each makes sense

| Runtime | Best for | Notes |
|---|---|---|
| **`tokio`** | almost everything — the default answer | multi-threaded work-stealing scheduler by default; the entire AWS SDK, most web frameworks (`axum`, `actix-web`'s newer versions), and most of the ecosystem is built against it |
| `async-std` | (largely historical) | designed to mirror `std`'s API shape closely; the project has scaled back — new projects should default to `tokio` unless there's a specific reason not to |
| `smol` | small, embeddable, minimal-dependency contexts | a lighter-weight executor for when `tokio`'s full feature set is more than you need |
| `embassy` | embedded, `no_std` targets (microcontrollers) | zero-heap-allocation executor — the shape needed when there's no OS underneath you at all |

**For everything in this repo, and for almost any general-purpose Rust
project, the answer is `tokio`.** It's what
[Rust in the AWS Ecosystem](../production/11-aws-ecosystem.md)'s SDK
examples already depend on, and it's the de facto standard the rest of
the ecosystem builds against — reach for an alternative only when you
have a concrete reason (embedded target, wanting a smaller dependency
footprint for a tiny CLI tool).

## Setting up `tokio`

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

`features = ["full"]` pulls in everything (networking, filesystem,
timers, sync primitives, macros) — fine for learning and most
applications; a production binary optimizing compile time/binary size
can instead enable only the specific features used (`["rt-multi-thread",
"macros", "net"]`, etc. — see [Cargo.toml anatomy](../workbook/10-cargo-modules-testing-macros.md#cargotoml-anatomy)
for the general `features = [...]` mechanism).

```rust
#[tokio::main]
async fn main() {
    // your async code
}
```

`#[tokio::main]` defaults to a **multi-threaded** runtime — it spins up
a thread pool (sized to your CPU count by default) and distributes
tasks across it, work-stealing style. This is worth knowing explicitly
because it means `tokio` tasks (chapter 4) aren't purely single-threaded
cooperative scheduling the way a naive mental model of "async = no
threads" might suggest — see
[Async vs. Threads](./05-async-vs-threads.md) for exactly how the two
combine.

```rust
#[tokio::main(flavor = "current_thread")]
async fn main() {
    // single-threaded — one thread, cooperative scheduling only
}
```

The single-threaded flavor is lighter weight (useful for a small CLI
tool, or tests) and, notably, is what removes the `Send` requirement on
spawned futures — see [Common Pitfalls](./06-common-pitfalls.md).

## The gotcha with no Python equivalent: runtime lock-in

Because non-blocking I/O (sockets, file handles, timers) has to be
implemented *by* a specific runtime — there's no `std`-level "non-
blocking read" to share — a `Future` built on `tokio`'s networking types
generally **cannot** be polled correctly by `async-std`'s executor, and
vice versa. This is the single sharpest edge Rust's "no batteries in
std" choice creates, and it has no counterpart in Python, where every
`async def` function works with any `asyncio`-compatible event loop by
construction.

Practically, this means:

- **Pick one runtime per project and stay on it.** Mixing `tokio`-based
  crates with `async-std`-based crates in the same binary is a common
  source of confusing runtime panics ("`no reactor running`" /
  "`there is no reactor running, must be called from the context of a
  Tokio 1.x runtime`") — the error is telling you a `Future` expecting
  `tokio`'s reactor got polled somewhere that isn't one.
- **When adding an async dependency, check what runtime it assumes.**
  Most crate docs state this explicitly ("built for `tokio`"); the AWS
  SDK, most database drivers (`sqlx`, `tokio-postgres`), and most web
  frameworks are `tokio`-based today, which is a large part of why
  `tokio` is the safe default choice.
- **CPU-bound and truly runtime-agnostic async code** (pure computation
  with no I/O) doesn't hit this problem — the lock-in specifically comes
  from I/O types (sockets, files, timers) that need a reactor to back
  them.

## Running blocking/sync code once inside an async `main`

A common early confusion: you're inside `#[tokio::main]`, but need to
call an ordinary synchronous function (a CPU-heavy computation, a
blocking library call). Never just call it directly on an async task —
see [Common Pitfalls](./06-common-pitfalls.md) for why — instead:

```rust
let result = tokio::task::spawn_blocking(|| {
    expensive_sync_computation()
}).await?;
```

`spawn_blocking` hands the closure to a **separate thread pool** `tokio`
maintains specifically for this, keeping your main async worker threads
free to keep servicing other `.await`-ing tasks. This is the bridge
between async code and the ordinary threaded world from
[Concurrency](../workbook/08-concurrency.md) — worth remembering exists
the first time you need to call something that doesn't `.await`.
