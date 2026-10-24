# 7. RTIC: the Alternative to Async

**RTIC** (Real-Time Interrupt-driven Concurrency) is embedded Rust's
other major concurrency framework — not async at all, but worth
understanding alongside [Embassy](./06-async-embedded-with-embassy.md)
since the two solve the same underlying problem (many logically
independent, hardware-triggered pieces of work on one core) with
opposite mental models.

## The model: priority-based preemption, no executor

```rust
#[rtic::app(device = pac)]
mod app {
    #[shared]
    struct Shared { counter: u32 }

    #[local]
    struct Local {}

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        (Shared { counter: 0 }, Local {})
    }

    #[task(binds = TIM2, shared = [counter], priority = 2)]
    fn tick(mut cx: tick::Context) {
        cx.shared.counter.lock(|c| *c += 1);
    }

    #[task(binds = USART1, priority = 1)]
    fn on_serial(_cx: on_serial::Context) {
        // lower priority — TIM2 can preempt this mid-execution
    }
}
```

Every RTIC "task" **is** a hardware interrupt handler
([Chapter 3](./03-interrupts-on-cortex-m.md)'s `#[interrupt]`, with
priorities and safe shared-state access layered on top by the
framework) — there's no `.await`, no `Future`, no executor polling
anything. The NVIC's own hardware priority mechanism *is* the
scheduler: a higher-priority task (`tick`, priority 2) genuinely
preempts a lower-priority one (`on_serial`, priority 1) already
running, mid-instruction, the same hardware-enforced nesting
[Chapter 3](./03-interrupts-on-cortex-m.md) noted is what the "nested"
in NVIC refers to.

## `#[shared]` + `.lock()` — the compile-time-checked version of a critical section

`cx.shared.counter.lock(|c| *c += 1)` isn't a runtime mutex
([Memory & Smart Pointers](../workbook/07-memory-and-smart-pointers.md)'s
`Mutex<T>`) — RTIC analyzes task priorities **at compile time** and
generates the minimum necessary critical section (briefly raising the
current priority, rather than a general lock/unlock with runtime
overhead) automatically, for exactly the data actually shared between
tasks that could preempt each other. This is
[Ownership, Borrowing & Lifetimes](../workbook/02-ownership-borrowing-lifetimes.md)'s
"many readers XOR one writer" rule, enforced by RTIC's macro at compile
time using the interrupt-priority structure itself as the proof,
instead of a runtime-checked `RefCell` or `Mutex`.

## RTIC vs. Embassy — the real choice

| | RTIC | Embassy |
|---|---|---|
| Concurrency model | Priority-preemptive interrupt handlers | Cooperative async tasks, `.await`-driven |
| Scheduling | Hardware (NVIC priorities) | Software (the async executor) |
| Shared state | `#[shared]` + compile-time-generated critical sections | Ordinary async-safe types (`Mutex`, channels — [Async Rust, Chapter 4](../async-rust/04-concurrency-patterns.md)'s primitives, embedded-adapted) |
| Best fit | Hard real-time guarantees, deterministic worst-case latency | I/O-heavy, many concurrent waits, less latency-critical |
| Mental model if you know... | OS interrupt handlers ([Assembly, Chapter 4](../asm/04-interrupts-and-exceptions.md)) | `tokio`/`asyncio` ([Async Rust](../async-rust/00-is-it-in-the-language-or-not.md)) |

Neither is strictly "better" — RTIC's hardware-priority preemption gives
stronger, more analyzable real-time guarantees (genuinely provable
worst-case latency, which safety-critical or hard-real-time embedded
work often requires); Embassy's async model is generally easier to
write and reason about for I/O-bound work with softer timing
requirements, and its `.await`-based code composes more naturally with
the rest of the async Rust ecosystem covered earlier in this book.
