# Assembly for OS Development: x86-64 & aarch64, from Rust

## Why this exists, and why Rust doesn't make it go away

Every layer of an OS kernel eventually bottoms out in instructions the
CPU executes that no safe-Rust abstraction covers: setting up a stack
before any function call is safe, switching privilege levels, enabling
an MMU, saving a task's registers to switch to another one. Rust gives
you `unsafe` and `core::arch::asm!` for exactly this — it doesn't
eliminate assembly from OS work, it gives you a typed, checked way to
drop into it only where truly necessary, and no further.

This section is a **side-by-side reference for the two architectures
already represented elsewhere in this book**:

- **aarch64** — [`hello-kernel`](../systems/07-hello-kernel-overview.md),
  a minimal freestanding kernel already in this repo.
- **x86-64** — [`OxideOS`](../oxideos/00-overview.md), a full-featured
  kernel with real interrupts, paging, syscalls, and a scheduler.

Every chapter here builds the general concept, then shows the actual
instructions on both architectures, pointing back at real code in one
or both of those projects wherever it already exists.

## Scope: OS development, not general assembly

This is not a general x86/ARM assembly tutorial — it covers
specifically the handful of things an OS kernel (or `hello-kernel`-style
freestanding binary) needs raw assembly for, and nothing else:

1. [Inline Assembly in Rust](./01-inline-asm-in-rust.md) — the `asm!`
   macro itself: syntax, operands, safety
2. [Registers & Calling Conventions](./02-registers-and-calling-conventions.md) —
   what's different enough between the two architectures to trip you up
3. [Privilege Levels & Mode Transitions](./03-privilege-levels-and-mode-transitions.md) —
   Rings vs Exception Levels
4. [Interrupts & Exceptions](./04-interrupts-and-exceptions.md) — IDT vs
   exception vector tables, and the actual ISR stub assembly
5. [Paging & MMU Setup](./05-paging-and-mmu-setup.md) — the exact
   instructions that turn on virtual memory
6. [Syscall Entry & Exit](./06-syscall-entry-exit.md) — `int`/`syscall`
   vs `svc`
7. [Context Switching](./07-context-switching.md) — the assembly that
   makes a scheduler possible
8. [Atomics & Memory Barriers](./08-atomics-and-memory-barriers.md) —
   the sharpest x86-vs-ARM gotcha of all
9. [Full Reference Checklist](./09-putting-it-together-checklist.md) —
   every piece above, one table, both architectures side by side

## Prerequisites

- [Ownership, Borrowing & Lifetimes](../workbook/02-ownership-borrowing-lifetimes.md) —
  `unsafe` doesn't turn these rules off, it just moves the compiler's
  proof obligation onto you
- [Dereferencing](../foundation/dereferencing.md) and
  [Chapter 1 of Systems](../systems/01-prerequisites-bits-bytes-and-addressing.md) —
  volatile memory-mapped registers, addresses, bytes
- Having read the [`hello-kernel` case study](../systems/07-hello-kernel-overview.md)
  helps a lot — its `_start`/UART code is referenced constantly here as
  the simplest possible real example of several of these techniques.
