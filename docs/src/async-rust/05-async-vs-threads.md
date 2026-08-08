# 5. Async vs. Threads — two different tools, not competing ones

[Concurrency](../workbook/08-concurrency.md) already covers OS threads
(`thread::spawn`, `Arc<Mutex<T>>`, channels). This chapter is the
question that naturally follows: given both exist, which do you
actually reach for?

## The one-sentence answer

**Threads are for using multiple CPUs at once (parallelism). Async is
for waiting on many things at once without blocking a thread for each
one (concurrency).** They solve different problems, and — as the last
section here covers — `tokio` actually uses *both* internally, so the
question isn't really "instead of," it's "which one matches what my
code is actually doing."

## Why an OS thread is the wrong tool for "wait on 10,000 sockets"

```rust
// one OS thread per connection — doesn't scale
for socket in incoming_connections {
    thread::spawn(move || handle(socket));
}
```

Every OS thread costs real memory for its stack (megabytes by default)
and real overhead for the kernel to schedule, regardless of whether
that thread is doing CPU work or just sitting blocked waiting for
network data. Ten thousand connections means ten thousand threads
mostly idle, waiting — exactly the scenario chapter 1 described async
solving: one thread can `.await` (and therefore stop actively occupying
the CPU) on thousands of sockets simultaneously, because the
suspend/resume machinery is a lightweight compiler-generated state
machine, not an OS-scheduled thread.

## Why async is the wrong tool for "sort a huge array"

```rust
async fn sort_it(mut v: Vec<i32>) -> Vec<i32> {
    v.sort();   // pure CPU work, no .await anywhere in here
    v
}
```

Putting `async` on this buys nothing — there's no I/O wait to yield
during, so it never actually suspends. Worse: if this runs directly on
one of `tokio`'s async worker threads, it **blocks that thread** for the
whole sort, starving every other task scheduled on it (see
[Common Pitfalls](./06-common-pitfalls.md)) — the opposite of async's
purpose. Real CPU-bound work belongs on an OS thread (via
[`std::thread`](../workbook/08-concurrency.md) directly, or
`tokio::task::spawn_blocking` from
[Choosing a Runtime](./02-choosing-a-runtime.md) if you're already
inside an async context and need to hand off computation without
blocking it).

## The decision, as a table

| Your code is mostly... | Reach for |
|---|---|
| Waiting on network calls, database queries, file I/O | `async`/`.await` |
| Crunching numbers, sorting, parsing, hashing — CPU-bound | OS threads ([Concurrency](../workbook/08-concurrency.md)), or `rayon` for data-parallel work |
| A mix of both | async for the I/O, `spawn_blocking` (or a dedicated thread pool) for the CPU-bound parts |

## How `tokio` actually combines both under the hood

The multi-threaded `tokio` runtime (chapter 2's default) is itself
built on a pool of OS threads — when you `.await` inside a `tokio`
task, you're not avoiding threads entirely, you're avoiding needing
**one thread per concurrent operation**. A handful of OS threads
(typically one per CPU core) can service tens of thousands of
concurrently-`.await`ing tasks, because each task only actually
*occupies* a thread while it's doing real, ready-to-run work — the
moment it hits `Poll::Pending` (chapter 1), that thread is freed to run
a different task. This is the actual mechanism behind async's headline
claim ("handle huge numbers of concurrent connections cheaply") — it's
not that threads disappeared, it's that the ratio of
concurrent-operations-to-threads went from 1:1 to many:1.

## Rule of thumb

- **Building a network service** (a web API, something calling AWS SDKs
  per [Rust in the AWS Ecosystem](../production/11-aws-ecosystem.md)) —
  async, almost certainly with `tokio`.
- **Writing a CLI tool that does one thing and exits** (like
  [`rgrep`](../../../use_cases/rgrep) as it currently stands) — plain
  synchronous code; async would add ceremony for zero benefit, since
  there's no concurrent I/O to overlap.
- **A CPU-bound batch job** (image processing, data crunching) —
  threads, or `rayon` for straightforward data-parallelism, not async.
- **Genuinely both** (a service that does some heavy CPU work per
  request) — async for the request handling/I/O, `spawn_blocking` or a
  dedicated thread pool for the CPU-heavy parts, exactly as chapter 2
  describes.
