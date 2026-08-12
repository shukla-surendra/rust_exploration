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
10. [Multicore & SMP](./10-multicore-and-smp.md) — who wakes the other
    cores, and why x86-64 needs a 16-bit trampoline to do it
11. [I/O Ports & MMIO](./11-io-ports-and-mmio.md) — how the CPU actually
    reaches a device's registers, tested against a real bare-metal boot

## A note on "firmware" — it's not one thing, and not every chip has it

"Firmware" gets used loosely throughout this section (ACPI tables,
PSCI calls in [Chapter 10](./10-multicore-and-smp.md), the boot process
in [Systems, Chapter 9](../systems/09-hello-kernel-boot-to-execution.md))
— worth pinning down what it actually refers to, since it's genuinely
several different things at different layers, not one:

- **Board/system firmware** (BIOS/UEFI on x86 boards, U-Boot/UEFI on
  many ARM boards) — lives in a separate flash chip **on the board**,
  not inside the CPU package at all. This is the "ROM, not RAM" the
  reset vector points at in
  [Systems, Chapter 9](../systems/09-hello-kernel-boot-to-execution.md).
- **A Boot ROM baked into the CPU/SoC silicon itself** — permanent,
  unchangeable, burned in at manufacture. On many ARM SoCs (though not
  most x86 desktop/server chips) this is the true first code executed,
  even before board firmware loads.
- **Microcode** — baked into the CPU die, translating the externally-
  visible instruction set into internal micro-ops; patchable via
  updates the BIOS/OS loads at boot, without a new chip.
- **Separate embedded management coprocessors** — Intel ME, AMD PSP:
  distinct small processors inside the same package as the main cores,
  running independent firmware, often orchestrating boot before the
  main cores even start.

**Microcontrollers, mostly, have none of this.** A Cortex-M chip
(see [Rust for Embedded Systems, Chapter 1](../embedded/01-no-std-and-the-embedded-toolchain.md))
typically has no BIOS/UEFI-equivalent — the program you flash *is* the
only thing that runs, starting almost immediately at reset. Some chips
still have a small immutable boot ROM purely for initial flashing/
recovery, but nothing resembling the multi-layer firmware stack above.

This matters for reading the rest of this section: when
[Chapter 10](./10-multicore-and-smp.md#step-1-discovery--firmware-tells-the-os-what-exists)
says "firmware tells the OS what cores exist," or PSCI is invoked as "a
firmware service," it's specifically the board/system-firmware layer
(or the boot-ROM/embedded-coprocessor layer providing PSCI on ARM) —
not something baked into every CPU unconditionally.

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
