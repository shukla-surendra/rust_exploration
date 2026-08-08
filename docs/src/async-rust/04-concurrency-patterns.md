# 4. Concurrency Patterns

**Python contrast:** this whole chapter maps almost one-to-one onto
`asyncio.create_task`, `asyncio.gather`, and `asyncio.wait` — if you've
used those, the shapes below will feel immediately familiar, just with
Rust's stricter ownership rules attached.

## `.await` alone is sequential, not concurrent

The single most common async mistake, in any language:

```rust
let a = fetch("a").await;   // waits for "a" to finish completely...
let b = fetch("b").await;   // ...THEN starts waiting for "b"
```

Two `.await`s in a row run **one after another**, not concurrently —
exactly as sequential as calling two ordinary blocking functions back to
back. `async` by itself buys you nothing here; concurrency has to be
requested explicitly, same as Python's `await a(); await b()` being
just as sequential as `asyncio.gather` is concurrent.

## `tokio::spawn` — an independent, concurrently-running task

```rust
let handle_a = tokio::spawn(fetch("a"));
let handle_b = tokio::spawn(fetch("b"));

let a = handle_a.await?;   // both "a" and "b" have been running concurrently since spawn
let b = handle_b.await?;
```

`tokio::spawn` is the async analog of
[`thread::spawn`](../workbook/08-concurrency.md#spawning-a-thread) —
hands a future to the runtime to run independently, returns a
`JoinHandle` you `.await` to get the result back. Directly comparable to
Python's `asyncio.create_task(...)`. Same `move` consideration as
`thread::spawn` applies if the future captures anything from the
enclosing scope (chapter 2's `Send`/`'static` requirements are the async
version of the same ownership rules from
[Ownership, Borrowing & Lifetimes](../workbook/02-ownership-borrowing-lifetimes.md)).

## `tokio::join!` — run several futures concurrently, wait for all

```rust
let (a, b) = tokio::join!(fetch("a"), fetch("b"));
```

Unlike `tokio::spawn`, `join!` doesn't hand futures off to run
independently on the runtime — it polls all of them concurrently *from
the current task*, no separate task/thread involved. Directly
`asyncio.gather(fetch("a"), fetch("b"))`. Use `join!` when you have a
**fixed, known-at-compile-time** number of futures to run together;
reach for `spawn` instead when you want each one to genuinely run as an
independent unit of work (e.g. surviving even if the spawning function
returns, or being cancelled independently).

## `tokio::select!` — race several futures, take the first to finish

```rust
tokio::select! {
    result = fetch("a") => println!("a finished first: {result}"),
    result = fetch("b") => println!("b finished first: {result}"),
    _ = tokio::time::sleep(std::time::Duration::from_secs(5)) => {
        println!("timed out");
    }
}
```

Runs all branches concurrently, and as soon as **one** completes, that
branch runs and every other branch's future is **dropped/cancelled**.
The timeout pattern above (racing real work against a `sleep`) is the
single most common use — directly `asyncio.wait_for(...)` in Python,
though `select!`'s "race arbitrary branches against each other" is more
general than `wait_for`'s "race one thing against a timeout"
specifically.

**Cancellation-safety gotcha, no direct Python parallel:** when a
`select!` branch loses the race, its future is dropped mid-flight.
Most futures handle this cleanly, but a future that was partway through
a multi-step operation (e.g. had written half of something to a socket)
can leave that operation incomplete with no chance to clean up — this is
called "cancellation safety," and it's worth being aware the concept
exists before relying on `select!` around anything with real side
effects. `tokio`'s own docs maintain a list of which of its own
operations are cancellation-safe for exactly this reason.

## `FuturesUnordered` — a dynamic, unknown-count collection of futures

`join!`/`select!` both need a fixed number of futures written out by
hand. For "run N futures concurrently, where N is only known at
runtime" (e.g. processing every message currently in an SQS batch —
see [Rust in the AWS Ecosystem](../production/11-aws-ecosystem.md)):

```rust
use futures::stream::{FuturesUnordered, StreamExt};

let mut tasks = FuturesUnordered::new();
for msg in messages {
    tasks.push(process_message(msg));
}

while let Some(result) = tasks.next().await {
    println!("{result:?}");
}
```

Results come back **in completion order**, not submission order —
same as `asyncio.as_completed(...)`. This is the shape for the "process
a batch concurrently" pattern that chapter 7 applies directly to the
SQS example already in this repo's AWS chapter.

## Which one to reach for

| Need | Use |
|---|---|
| A fixed, small number of independent async operations, want all results | `tokio::join!` |
| An operation that should keep running independently of the current scope | `tokio::spawn` |
| "Whichever finishes first" / a timeout | `tokio::select!` |
| A runtime-determined number of concurrent operations | `FuturesUnordered` |
| Just need one thing to finish before moving on | plain `.await`, no concurrency needed |
