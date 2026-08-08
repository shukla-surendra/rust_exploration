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

## This section assumes ARM Cortex-M specifically — here's why

"Microcontroller" covers several genuinely different, mutually
incompatible instruction sets, not one — worth surveying before diving
into chapters that otherwise silently assume Cortex-M throughout:

| Chip / family | Instruction set | Cores | Rust support |
|---|---|---|---|
| STM32 (ST) | ARM Cortex-M (M0/M0+/M3/M4/M7/M33) | 1 (2, asymmetric, on H745/755) | First-class, upstream `rustup target add` |
| RP2040 (Pico) | ARM Cortex-M0+ | 2, symmetric | First-class, upstream |
| RP2350 (Pico 2) | Cortex-M33 *or* RISC-V (Hazard3), same silicon | 2 | First-class, upstream (both options) |
| nRF52/nRF51 (Nordic) | ARM Cortex-M4/M0 | 1 | First-class, upstream |
| ESP32 (original, S2, S3) | **Xtensa** LX6/LX7 | 2 (S2: 1) | **Fork only** — `esp-rs`'s patched compiler, installed via `espup`, not `rustup` |
| ESP32-C3, C6, H2 | **RISC-V** (RV32IMC) | 1 | First-class, upstream |
| AVR (classic Arduino Uno/Nano) | AVR, 8-bit | 1 | Nightly-only, `-Z build-std`, `avr-hal` |
| PIC (Microchip) | PIC, proprietary (PIC32: MIPS) | 1 | No practical ecosystem |
| MSP430 (TI) | MSP430, 16-bit | 1 | No practical ecosystem |
| 8051 (legacy/cheap designs) | 8051, 8-bit | 1 | No practical ecosystem |

**ARM Cortex-M is this section's example architecture specifically
because it has the best Rust support by a wide margin** — fully
upstream in `rustc`/LLVM, no forked toolchain, the deepest
`embedded-hal`/PAC/HAL ecosystem. Everything in
[Chapter 3](./03-interrupts-on-cortex-m.md) (the NVIC) and the `asm!`
examples throughout are Cortex-M-specific for this reason; a RISC-V
microcontroller (an ESP32-C3, say) uses a genuinely different interrupt
mechanism and a different `-rt` crate (`riscv-rt` instead of
`cortex-m-rt`), though the higher layers — `embedded-hal` traits,
`heapless`, even Embassy — are largely architecture-agnostic and work
similarly across both.

**Most microcontrollers are single-core** — the default this whole
section assumes, matching `cortex-m-rt`'s single `#[entry]` function
and one NVIC. Dual-core (RP2040/2350, ESP32/S3) is real but still a
minority; when it shows up, starting the second core is usually a
single vendor-SDK call ("here's your stack, here's your entry point"),
**not** anything resembling
[Assembly, Chapter 10](../asm/10-multicore-and-smp.md)'s x86-64
INIT-SIPI-SIPI real-mode trampoline — there's no privilege levels, no
legacy boot mode, and often no OS at all to satisfy, so the whole
reason that trampoline exists on x86-64 simply doesn't apply. (A few
safety-critical automotive/industrial chips, e.g. Infineon's AURIX
TriCore family, have 3+ cores running in **lockstep** — executing
identical code redundantly for fault detection — a different purpose
entirely from the SMP parallelism Assembly Chapter 10 covers.)

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
