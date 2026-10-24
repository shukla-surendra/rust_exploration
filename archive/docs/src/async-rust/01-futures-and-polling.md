# 1. Futures & Polling — what `.await` actually does

**Python contrast:** a Python coroutine (`async def`) and a Rust
`Future` play the same role, but Rust's version is a plain, inert value
until something polls it — closer to a generator you manually call
`.send(None)` on than to `asyncio`'s coroutines, which the event loop
drives for you without you ever seeing the mechanism. This chapter is
that mechanism, made visible.

## `Future` is one trait, and it's small

```rust
trait Future {
    type Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

enum Poll<T> {
    Ready(T),
    Pending,
}
```

That's the entire contract. A `Future` is anything with a `poll`
method that either says "I'm done, here's the value" (`Poll::Ready(v)`)
or "not yet, ask me again later" (`Poll::Pending`). **Nothing about this
is async-runtime-specific** — it's a plain trait in `std`, same
category as `Iterator` or `Display` from
[Traits](../foundation/traits.md). What makes it *feel* async is purely
how it gets called.

## `async fn` is sugar for a hand-written state machine

```rust
async fn add_one(x: i32) -> i32 {
    x + 1
}
```

desugars, roughly, into something you could write by hand:

```rust
struct AddOneFuture { x: i32 }

impl Future for AddOneFuture {
    type Output = i32;
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<i32> {
        Poll::Ready(self.x + 1)   // no .await inside — always finishes on first poll
    }
}
```

For a function with an `.await` inside, the compiler generates something
closer to an enum with one variant per "suspend point" — every place the
function might have to pause and come back later:

```rust
async fn fetch_and_double(url: &str) -> i32 {
    let n = fetch_number(url).await;   // <- a suspend point
    n * 2
}

// roughly desugars to:
enum FetchAndDoubleFuture {
    Start { url: String },
    WaitingOnFetch { inner: FetchNumberFuture },
    Done,
}
```

Each `poll()` call either makes progress and returns `Poll::Pending`
(because the inner `fetch_number(url)` future isn't ready yet), or
reaches the end and returns `Poll::Ready(n * 2)`. **This is why async
Rust has no per-`.await` heap allocation or OS thread by default** — the
whole chain of suspend points compiles into one flat, stack-sized state
machine, unlike a Python coroutine (which the interpreter manages via
its own frame objects) or an OS thread (which needs a whole stack
allocated up front — see [Concurrency](../workbook/08-concurrency.md)).
This compactness is the actual payoff of Rust choosing "just a trait,
no built-in runtime" — it's what lets a single `tokio` process run tens
of thousands of concurrent `.await`s cheaply, where the same count of OS
threads would exhaust memory well before that.

## Who calls `poll()`, and how "waiting efficiently" works

Nobody calls `.poll()` by hand in ordinary code — the **executor**
(`tokio`, from [Choosing a Runtime](./02-choosing-a-runtime.md)) does,
and it does something specific with `Poll::Pending`:

1. Executor calls `future.poll(cx)`.
2. Future does some work, hits an operation that isn't ready yet (e.g.
   a socket with no data to read), and returns `Poll::Pending` —
   crucially, **after registering a `Waker`** (from the `Context`
   passed in) with whatever it's waiting on (the OS's I/O event
   notification, a timer, etc.).
3. The executor, seeing `Pending`, **does not poll that future again
   yet** — it goes and makes progress on some *other* ready future
   instead. This is the entire "waiting efficiently" story: the thread
   isn't blocked idle, it's doing other useful work.
4. Later, the thing being waited on becomes ready (data arrives) and
   calls `waker.wake()` — this tells the executor "poll this future
   again, it might make progress now."
5. The executor polls it again; either another `Pending` (still
   waiting on something else) or finally `Poll::Ready(value)`.

This wake-driven design is why async is specifically good for
**I/O-bound waiting** and not a substitute for CPU parallelism (see
[Async vs. Threads](./05-async-vs-threads.md)) — the entire mechanism
exists to avoid a thread sitting idle while waiting on something
external, not to run more computation simultaneously.

## `Pin` — the one piece worth knowing exists, not mastering

You'll see `Pin<&mut Self>`/`Pin<Box<dyn Future>>` in error messages and
some function signatures long before you need to write your own `Future`
by hand. The one-sentence version: because the compiler-generated state
machine above can contain **self-references** (a suspended future
holding a reference to its own local data across an `.await` point),
that value must not be allowed to *move in memory* after it starts being
polled — moving it would invalidate those internal references. `Pin`
is the type-level promise "this value will not move again." Ordinary
`async fn`/`.await` usage (everything in this tutorial) never requires
you to write `Pin` yourself — it shows up as `Box::pin(...)` occasionally
when storing a `Future` in a struct field or a `Vec`, which
[Concurrency Patterns](./04-concurrency-patterns.md) touches on for
`FuturesUnordered`.

## Why this matters even though you'll never write `poll()` by hand

Every practical async gotcha in
[Common Pitfalls](./06-common-pitfalls.md) — accidentally blocking the
executor, an `.await` that "does nothing" because you forgot it,
confusing compiler errors about `Send`/lifetimes — traces back to this
mechanism. Knowing that `.await` means "suspend here, let the executor
run something else, resume when woken" is what makes those error
messages and bugs legible instead of mysterious.
