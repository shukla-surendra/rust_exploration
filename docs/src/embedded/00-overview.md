# Rust for Embedded Systems

## How this differs from everything else in this book

[`hello-kernel`](../systems/07-hello-kernel-overview.md) and
[OxideOS](../oxideos/00-overview.md) target **application processors** —
CPUs with an MMU, running a full OS that itself hosts other programs.
"Embedded" here means something more specific: **microcontrollers** —
chips like the ARM Cortex-M family, usually with no MMU, a few hundred
KB of RAM at most, no OS underneath your code at all (your program *is*
the entire system), and often powered by a coin cell or running for
years unattended.

The overlap with everything in [Assembly for OS Development](../asm/00-overview.md)
and [Systems From First Principles](../systems/00-overview.md) is real
— `#![no_std]`, raw MMIO register access, interrupt vector tables,
`unsafe` — but the specifics, the ecosystem, and the actual day-to-day
workflow are different enough to deserve their own section rather than
being folded into either.

## The layered ecosystem, and where each piece sits

```
Your application code
    ↓ uses
Board Support Crate (BSP)      — e.g. rp-pico, stm32f4xx-hal's board profiles
    ↓ built on
HAL (Hardware Abstraction Layer) — e.g. stm32f4xx-hal, rp2040-hal
    ↓ built on
PAC (Peripheral Access Crate)   — e.g. stm32f4, rp2040-pac (generated from vendor SVD files)
    ↓ built on
cortex-m-rt + cortex-m           — startup code, linker script, interrupt vector table
    ↓ built on
core::arch::asm! / volatile MMIO — the same primitives Assembly for OS Development covers
```

`hello-kernel`'s hand-rolled `_start` and `uart_putc` (raw
`read_volatile`/`write_volatile` at a hardcoded address) are, in this
stack, doing by hand exactly what `cortex-m-rt` and a PAC automate for
you — see [Chapter 1](./01-no-std-and-the-embedded-toolchain.md) and
[Chapter 2](./02-pac-hal-and-registers.md) for that comparison made
explicit. This is deliberate: understanding `hello-kernel`'s manual
version first makes the embedded ecosystem's generated code legible
instead of magic.

## Chapters

1. [`no_std` and the Embedded Toolchain](./01-no-std-and-the-embedded-toolchain.md) —
   `cortex-m-rt`, `memory.x`, flashing real hardware
2. [PACs, HALs & Registers](./02-pac-hal-and-registers.md) — the
   generated, type-safe version of `hello-kernel`'s raw pointer writes
3. [Interrupts on Cortex-M](./03-interrupts-on-cortex-m.md) — the NVIC,
   and why it's simpler than [x86/aarch64 interrupts](../asm/04-interrupts-and-exceptions.md)
4. [`embedded-hal` & Drivers](./04-embedded-hal-and-drivers.md) —
   chip-agnostic peripheral traits
5. [Memory Constraints & `heapless`](./05-memory-constraints-and-heapless.md) —
   living without a heap, or without much of one
6. [Async Embedded with Embassy](./06-async-embedded-with-embassy.md) —
   the exact `Future`/executor model from
   [Async Rust](../async-rust/00-is-it-in-the-language-or-not.md),
   applied to a microcontroller
7. [RTIC: the Alternative](./07-rtic-alternative.md) — priority-based
   preemptive concurrency without an RTOS
8. [Hands-On: Blinky and Beyond](./08-hands-on-blinky-and-beyond.md) —
   a full worked example, runnable under QEMU
9. [Embedded vs. OS Dev: Cheat Sheet](./09-embedded-vs-os-dev-cheat-sheet.md) —
   the whole comparison, one table

## Prerequisites

- [Ownership, Borrowing & Lifetimes](../workbook/02-ownership-borrowing-lifetimes.md)
  and [Traits](../foundation/traits.md) — the type-safety tricks PACs/HALs
  use (chapter 2) lean on generics and trait bounds heavily.
- [Chapter 1 of Systems](../systems/01-prerequisites-bits-bytes-and-addressing.md) —
  volatile, addresses, bytes.
- Having read [`hello-kernel`](../systems/07-hello-kernel-overview.md)
  and, ideally, [Assembly for OS Development](../asm/00-overview.md) —
  not strictly required, but every chapter here contrasts against them.
