# 6. Common Pitfalls

Every one of these traces back to a mechanism already covered in this
tutorial — listed here as a lookup table for "why is my async code
doing something weird," since that's how you'll actually encounter
them.

## 1. Forgetting `.await` — a future that silently does nothing

```rust
async fn save_to_db(record: Record) { /* ... */ }

fn handler() {
    save_to_db(record);   // BUG: compiles, does nothing — no .await
}
```

Already covered in [Chapter 0](./00-is-it-in-the-language-or-not.md) —
worth restating here because it's the single most common async bug in
Rust: calling an `async fn` without `.await`ing (or otherwise polling)
it constructs a `Future` and immediately drops it, unexecuted. The
compiler does emit `#[must_use]` warnings for unused futures in most
cases (`warning: unused implements 'Future' that must be used`) — never
ignore that specific warning.

## 2. Blocking the executor with synchronous work

```rust
#[tokio::main]
async fn handler() {
    std::thread::sleep(std::time::Duration::from_secs(5));   // BUG in async code
    // or: std::fs::read_to_string(...) — blocking I/O
    // or: a long, uninterrupted CPU-bound loop
}
```

Straight from [Async vs. Threads](./05-async-vs-threads.md): any of
these **block the entire OS thread** they're running on for their
duration — and since that thread is one of `tokio`'s worker threads,
every *other* task scheduled on it is frozen too, not just this one.
This is a categorically worse failure mode than the equivalent mistake
in Python (`asyncio` has the same footgun with a blocking call inside a
coroutine, single-threaded-event-loop version), because a busy `tokio`
runtime might only have a handful of worker threads serving thousands of
tasks — blocking one is proportionally much more damaging.

**Fixes:**

| Blocking thing | Async replacement |
|---|---|
| `std::thread::sleep` | `tokio::time::sleep(...).await` |
| `std::fs::read_to_string` | `tokio::fs::read_to_string(...).await` |
| a blocking network call / blocking library with no async version | `tokio::task::spawn_blocking(...)` (chapter 2) |
| genuine CPU-bound work | `spawn_blocking`, or move it to a real OS thread |

## 3. Holding a `std::sync::Mutex` guard across an `.await`

```rust
let guard = std_mutex.lock().unwrap();
some_async_call().await;   // BUG: guard still held while suspended
drop(guard);
```

A `std::sync::MutexGuard` held across a suspend point keeps the lock
held for the *entire* time this task is paused waiting on
`some_async_call` — potentially a long time, and worse, on the
multi-threaded runtime this can deadlock outright (the guard isn't
`Send`, so the compiler often catches this specific shape as a hard
error rather than a silent performance problem — see chapter 3's `Send`
section). The fix: use `tokio::sync::Mutex` instead for anything locked
across an `.await`, or restructure so the lock is dropped before the
`.await` (compute what you need from the guarded data, drop the guard,
*then* `.await`).

```rust
let value = { let guard = std_mutex.lock().unwrap(); guard.clone() };  // guard dropped here
some_async_call().await;   // no lock held during the wait
```

## 4. Assuming `.await` order is execution order

```rust
let a = tokio::spawn(fetch("a"));
let b = tokio::spawn(fetch("b"));
// both are already running concurrently the moment spawn() is called —
// awaiting a first doesn't mean a "starts first" in any meaningful sense
```

Spawned tasks begin running as soon as they're scheduled, not when
you `.await` their handle — `.await` on a `JoinHandle` just waits for
the *result*, it doesn't control when the work started. This rarely
causes bugs but frequently causes wrong mental models about ordering,
especially coming from sequential `.await` chains (pitfall-free, but a
different execution shape than spawned tasks).

## 5. `Send`/`'static` errors that seem to come from nowhere

Covered in depth in chapter 3 — the short version for a lookup table:
`tokio::spawn` requires `Send + 'static`. If you're passing a reference
(`&T`) or a non-thread-safe type (`Rc<T>`, a `RefCell` guard) into a
spawned future, expect a compiler error naming exactly which piece
violates the bound. The fix is almost always "own the data instead of
borrowing it" (`String` instead of `&str`, `Arc` instead of `Rc`) — the
same ownership discipline from
[Ownership, Borrowing & Lifetimes](../workbook/02-ownership-borrowing-lifetimes.md),
just enforced at a thread-safety boundary instead of a single-threaded
borrow-checker boundary.

## 6. Mixing runtimes

Covered in chapter 2 — restated here because the *symptom* doesn't
obviously point at the *cause*: a panic mentioning "no reactor running"
or "must be called from the context of a Tokio runtime" almost always
means a `tokio`-dependent type (a socket, a timer) got constructed or
polled outside of an actual `tokio` runtime context — commonly from
mixing a `tokio`-based dependency with a non-`tokio` executor, or
constructing `tokio` I/O types before `#[tokio::main]`'s runtime has
actually started.
