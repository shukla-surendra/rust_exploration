# 7. Practical Example: Making the AWS Code Actually Concurrent

[Rust in the AWS Ecosystem](../production/11-aws-ecosystem.md)'s SQS
example processes messages **sequentially** — each message is fully
handled (printed, deleted) before the next one starts, even though
nothing about the work actually requires that order. This chapter
applies chapters 1–6 to fix exactly that, which is the real payoff of
having read this far: this is what async is *for*.

## The sequential version, annotated

```rust
for msg in resp.messages() {
    println!("{}", msg.body().unwrap_or(""));

    client
        .delete_message()
        .queue_url(queue_url)
        .receipt_handle(msg.receipt_handle().unwrap())
        .send()
        .await?;   // <- waits for THIS delete to finish before the loop continues
}
```

Per [Concurrency Patterns](./04-concurrency-patterns.md)'s opening
point: a plain `.await` inside a loop is exactly as sequential as a
blocking call would be. If each `delete_message` call takes, say,
50ms of network round-trip, ten messages take 500ms total — even though
every single one of those calls is *waiting on the network*, the exact
scenario async exists to overlap.

## Version 1: fixed-size concurrency with `join!`

Fine for a small, known batch (SQS caps a single `receive_message` at 10
messages anyway — see the original example):

```rust
use futures::future::join_all;

let deletes = resp.messages().iter().map(|msg| {
    client
        .delete_message()
        .queue_url(queue_url)
        .receipt_handle(msg.receipt_handle().unwrap())
        .send()
});

let results = join_all(deletes).await;   // all 10 in flight concurrently

for result in results {
    result?;   // surface any individual failure
}
```

`join_all` (from the `futures` crate) is `join!`'s dynamic-count sibling
— same "run everything concurrently, wait for all to finish" idea from
[Concurrency Patterns](./04-concurrency-patterns.md), but taking a
runtime-sized collection of futures instead of a fixed, written-out
list. Ten network round-trips now overlap instead of stacking — total
time closer to one round-trip's worth (~50ms) instead of ten.

## Version 2: process-as-they-complete with `FuturesUnordered`

If processing each message involves real work you want to start acting
on as soon as it's ready (not just fire-and-collect, like the deletes
above), reach for the pattern from chapter 4 directly:

```rust
use futures::stream::{FuturesUnordered, StreamExt};

let mut tasks = FuturesUnordered::new();

for msg in resp.messages() {
    let client = client.clone();   // aws-sdk clients are cheap to clone — see below
    let receipt = msg.receipt_handle().unwrap().to_string();
    let body = msg.body().unwrap_or("").to_string();

    tasks.push(async move {
        println!("processing: {body}");
        // ... your actual per-message work here, e.g. write to a database ...

        client
            .delete_message()
            .queue_url(QUEUE_URL)
            .receipt_handle(receipt)
            .send()
            .await
    });
}

while let Some(result) = tasks.next().await {
    if let Err(e) = result {
        eprintln!("failed to process/delete a message: {e}");
    }
}
```

Notice the `.clone()` and `.to_string()` calls — this is chapter 3's
`Send`/`'static` requirement showing up directly: each task pushed into
`FuturesUnordered` needs to own everything it touches (an owned
`String` instead of borrowing `msg`, a cloned `client` instead of a
shared reference into the loop), the same ownership discipline as
`tokio::spawn`. **AWS SDK `Client` types are cheap to clone** —
internally they're an `Arc`-wrapped handle (see
[Memory & Smart Pointers](../workbook/07-memory-and-smart-pointers.md)),
so `client.clone()` here is bumping a reference count, not duplicating a
real connection pool.

## Why this isn't the default — and when sequential is actually correct

- **Ordering matters** for some workloads (must process message N before
  N+1) — sequential `.await` is the *correct* choice there, not a
  missed optimization. Don't reach for concurrency just because it's
  available; reach for it when the operations are genuinely independent.
- **Rate limits** — AWS APIs have per-account/per-resource request
  limits. Blasting out unlimited concurrent calls (`join_all` over an
  unbounded collection) can trip throttling that a sequential loop
  never would. A `tokio::sync::Semaphore` capping concurrent in-flight
  requests to some fixed number is the common middle ground — worth
  knowing exists once a batch grows beyond SQS's small 10-message cap
  used above.
- **Error handling gets more nuanced** — a sequential loop can `?` out
  immediately on the first failure; a concurrent batch has to decide
  whether one failed message should abort the whole batch or just be
  logged and skipped (the `if let Err(e)` above chooses "log and
  continue," a deliberate choice, not the only correct one).

## The throughline for this whole tutorial

Nothing in this chapter is a new concept — it's chapters 1 through 6
applied to code you already had. That's deliberate: async Rust's real
learning curve is entirely in the six chapters before this one (what a
`Future` is, why a runtime is required, `Send`/`'static`, the
concurrency primitives, the pitfalls); once those are solid, "make this
existing sequential AWS code concurrent" is a small, mechanical
transformation, not a new topic.
