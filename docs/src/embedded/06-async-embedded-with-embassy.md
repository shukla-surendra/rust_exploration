# 6. Async Embedded with Embassy

[Async Rust, Chapter 0](../async-rust/00-is-it-in-the-language-or-not.md)
established the split that makes this chapter possible: `async`/`.await`
and the `Future` trait are part of the *language* (`core`, available
even in `#![no_std]`); only the **executor** is external, and different
executors suit different environments. Embassy is that executor for
microcontrollers — and it's worth understanding as a direct proof of
[Async Rust, Chapter 2](../async-rust/02-choosing-a-runtime.md)'s claim
that `std` deliberately doesn't ship one, *because* environments this
different genuinely need different implementations.

## Why async fits embedded surprisingly well

[Async Rust, Chapter 5](../async-rust/05-async-vs-threads.md) drew the
line: async is for *waiting on many things without blocking a thread
per wait*. A microcontroller reading a sensor over I2C, waiting on a
button press, and blinking an LED on a timer is exactly that shape —
several independent, mostly-idle, interrupt-driven waits — and
critically, most Cortex-M chips have **no OS, no threads, and often
just one core**, so [`tokio`](../async-rust/02-choosing-a-runtime.md)'s
whole multi-threaded-work-stealing model is both unavailable and
unnecessary. Embassy's executor is **single-threaded and interrupt-
driven**: a task suspends at `.await`, and instead of a thread pool
polling it, the *hardware peripheral itself* fires an interrupt
([Chapter 3](./03-interrupts-on-cortex-m.md)'s NVIC) that wakes exactly
that task — the `Waker` mechanism from
[Async Rust, Chapter 1](../async-rust/01-futures-and-polling.md) mapped
directly onto real hardware interrupts instead of an OS's I/O
notification API.

## What it looks like

```rust
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::Timer;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    spawner.spawn(blink_task()).unwrap();
    loop {
        Timer::after_millis(1000).await;
    }
}

#[embassy_executor::task]
async fn blink_task() {
    let mut led = /* ... from a HAL, Chapter 2 ... */;
    loop {
        led.set_high();
        Timer::after_millis(500).await;
        led.set_low();
        Timer::after_millis(500).await;
    }
}
```

`#[embassy_executor::main]` is `#[tokio::main]`'s exact counterpart
([Async Rust, Chapter 2](../async-rust/02-choosing-a-runtime.md)) —
same job (start a runtime, drive the top-level future), different
runtime underneath. `spawner.spawn(...)` is
[`tokio::spawn`](../async-rust/04-concurrency-patterns.md)'s
counterpart — an independent, concurrently-running task, except here
"concurrent" means genuinely interrupt-driven on one core, not
work-stolen across a thread pool. `Timer::after_millis(...).await`
doesn't busy-wait — it suspends the task and lets the executor run
other tasks (or put the whole chip into a low-power sleep state) until
a hardware timer interrupt fires and wakes it, the literal embedded
equivalent of `hello-kernel`'s `wfe` idle loop
([Systems, Chapter 9](../systems/09-hello-kernel-boot-to-execution.md)),
but resumed into a specific suspended task instead of just looping.

## The pitfalls chapter still applies, mostly unchanged

[Async Rust, Chapter 6](../async-rust/06-common-pitfalls.md)'s "blocking
the executor" warning is *more* dangerous here, not less — a single-
threaded, single-core embedded executor has nowhere else to run other
tasks at all while one is blocked, unlike `tokio`'s thread pool, which
can at least keep other worker threads busy. A `for` loop doing a tight
poll instead of `.await`ing a proper async peripheral driver stalls
*everything* on the chip, including interrupt-driven blinking that
should have kept running.

## Why not just use threads instead?

[Async vs. Threads](../async-rust/05-async-vs-threads.md)'s core
question, answered concretely for this environment: most Cortex-M
chips have no OS to schedule OS-level threads on in the first place —
"threads" in the [Concurrency](../workbook/08-concurrency.md) sense
don't exist here at all without first building or including an RTOS.
Async gives you genuine concurrency (many logically-independent,
interrupt-woken tasks) on bare hardware with **zero heap allocation**
required (per [Chapter 5](./05-memory-constraints-and-heapless.md) —
Embassy's executor is statically sized, no dynamic task allocation
needed) — which is precisely why it, rather than a full RTOS port, has
become one of the two dominant concurrency answers in modern embedded
Rust, alongside the alternative covered next.
