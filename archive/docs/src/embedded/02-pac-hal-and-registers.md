# 2. PACs, HALs & Registers: Type-Safe MMIO

## What `hello-kernel` does by hand

```rust
const UART_DR: usize = 0x0900_0000;
core::ptr::write_volatile(UART_DR as *mut u32, byte as u32);
```

Exactly correct, and exactly what
[Systems, Chapter 10](../systems/10-hello-kernel-uart-and-panics.md)
explains — but nothing stops you from writing to the wrong address, the
wrong width, or a bit pattern the hardware doesn't accept; the type
system doesn't know `0x0900_0000` means anything. The embedded
ecosystem's answer is to generate a typed wrapper around exactly this
pattern, automatically, per chip.

## PAC — Peripheral Access Crate, generated from the vendor's own spec

Chip vendors publish an **SVD** (System View Description) file — an XML
description of every peripheral, register, and bit field on a specific
chip. `svd2rust` compiles that XML into a Rust crate:

```rust
use stm32f4xx_hal::pac;

let peripherals = pac::Peripherals::take().unwrap();
peripherals.GPIOA.moder.modify(|_, w| w.moder5().output());
peripherals.GPIOA.odr.modify(|_, w| w.odr5().set_bit());
```

`peripherals.GPIOA.moder` is a real struct at a real memory address
(the generated code performs the identical `read_volatile`/
`write_volatile` `hello-kernel` does by hand underneath) — but
`.moder5().output()` is a method that **only compiles if that bit
pattern is valid for that register**, because it's generated directly
from the vendor's own definition of what that register accepts. A typo
that would silently write garbage to a hardware register in
`hello-kernel`'s raw-pointer style becomes a compile error here.

**`Peripherals::take()` returning an `Option`, taken exactly once** is
deliberate, not incidental: it enforces at compile time that only one
piece of code in the whole program can hold `GPIOA` at a time — the
`Ownership, Borrowing & Lifetimes` rules
([Workbook, Chapter 2](../workbook/02-ownership-borrowing-lifetimes.md))
applied directly to a *hardware register*, preventing two unrelated
parts of your program from fighting over the same peripheral the same
way Rust prevents two mutable references to the same value.

## HAL — Hardware Abstraction Layer, one level friendlier

```rust
use stm32f4xx_hal::{pac, prelude::*};

let dp = pac::Peripherals::take().unwrap();
let gpioa = dp.GPIOA.split();
let mut led = gpioa.pa5.into_push_pull_output();
led.set_high();
```

The PAC exposes registers as the vendor defined them — a HAL builds
higher-level types on top (`led.set_high()` instead of manually
computing which bit of `ODR` corresponds to pin 5) and, crucially,
implements the shared **`embedded-hal`** trait set
([Chapter 4](./04-embedded-hal-and-drivers.md)) so driver crates written
against those traits work against *any* chip's HAL, not just this one.

## The layering, restated against `hello-kernel`

| Layer | What it provides | `hello-kernel`'s equivalent |
|---|---|---|
| PAC | Typed register access, generated from vendor SVD | hand-written `const UART_DR: usize = ...` + raw `read_volatile`/`write_volatile` |
| HAL | Friendly, chip-specific typed API (`led.set_high()`) | `uart_putc`/`uart_puts` — hand-written, one-off |
| `embedded-hal` traits | Chip-*agnostic* API (`OutputPin::set_high()`) | *(none — `hello-kernel` is chip-specific by design, a teaching example)* |

`hello-kernel` intentionally stops at the "HAL" row, hand-rolled and
specific to one machine (QEMU's `virt` UART) — exactly right for a
minimal teaching kernel. A real embedded project reaches for the full
stack specifically because it needs to run on real, varied hardware,
and wants driver code (Chapter 4) that doesn't have to be rewritten per
chip.
