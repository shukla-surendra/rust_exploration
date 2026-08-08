# 1. `no_std` and the Embedded Toolchain

## The same `#![no_std]`/`#![no_main]` starting point

```rust
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

#[entry]
fn main() -> ! {
    loop {}
}
```

Identical opening to `hello-kernel` (see
[Systems, Chapter 7](../systems/07-hello-kernel-overview.md)) — no
standard library, no C runtime. The differences start immediately after:
`hello-kernel` writes `_start` and the linker script by hand; here,
`cortex_m_rt::entry` and a separate `memory.x` file do that work for
you, generated once per chip family rather than once per project.

## `cortex-m-rt` — `hello-kernel`'s `_start` + `linker.ld`, generalized

`cortex-m-rt` provides, as a dependency rather than hand-written code:

- **The reset handler** — the Cortex-M equivalent of `_start`, doing
  the same job (`hello-kernel`'s `_start` sets up the stack by hand;
  Cortex-M's hardware exception model actually loads the *initial stack
  pointer* directly from the vector table's first entry — see
  [Chapter 3](./03-interrupts-on-cortex-m.md) — so `cortex-m-rt`'s reset
  handler's first job is instead zeroing `.bss` and copying `.data`'s
  initial values from flash into RAM, the "startup" step
  [Systems, Chapter 8](../systems/08-hello-kernel-build-and-linking.md)'s
  `.bss` section noted `hello-kernel` doesn't yet bother with).
- **The `#[entry]` attribute** — marks your `fn main() -> !` as the
  function to call after that startup work completes; equivalent to
  `hello-kernel`'s `bl kernel_main`.
- **A default linker script shape** (`link.x`) that your project's
  `memory.x` plugs chip-specific addresses into (below) — same role as
  `hello-kernel`'s `linker.ld`, but split so the *chip-specific* part
  (how much flash/RAM, at what addresses) is the only thing you write
  per project.

## `memory.x` — just the addresses, not the whole layout

```
/* memory.x */
MEMORY
{
    FLASH : ORIGIN = 0x08000000, LENGTH = 512K
    RAM   : ORIGIN = 0x20000000, LENGTH = 128K
}
```

Compare to `hello-kernel`'s `linker.ld`
([Systems, Chapter 8](../systems/08-hello-kernel-build-and-linking.md)),
which spells out every section's placement by hand. `cortex-m-rt`'s
`link.x` (pulled in automatically) already knows the *shape* every
Cortex-M program needs (`.vector_table`, `.text`, `.rodata`, `.data`,
`.bss`, stack) — your `memory.x` only supplies the two numbers that
differ per chip: where flash starts and how big it is, same for RAM.
This is the ecosystem generalizing exactly what `hello-kernel` did by
hand for one specific address (`0x40080000`) into "tell us your chip's
two address ranges, we'll handle the rest."

## Flashing and running on real hardware

Unlike `hello-kernel` (which only ever runs under QEMU), embedded
development's default target is **real silicon** — a devboard sitting
on your desk over USB. `probe-rs` is the modern standard tool:

```toml
# .cargo/config.toml
[target.thumbv7em-none-eabihf]
runner = "probe-rs run --chip STM32F411CEUx"
```

```sh
cargo run --release
```

Exactly the same `cargo run` → `runner` wiring
[`hello-kernel`'s own `.cargo/config.toml`](../systems/08-hello-kernel-build-and-linking.md)
uses for QEMU — here `probe-rs` erases flash, writes your compiled
binary to it over a debug probe (many boards have one built in, e.g.
ST-Link on STM32 Nucleo/Discovery boards), resets the chip, and
optionally streams `defmt`/RTT log output back over the same USB
connection — a debugger and a serial console in one tool, with no
separate flashing utility needed.

**QEMU exists here too** — for chips it emulates (several Cortex-M
targets, notably via the `mps2-an385`/`lm3s6965evb`/similar machine
types), letting you follow this whole section without owning hardware,
the same workflow as `hello-kernel`'s aarch64 QEMU setup. Chapter 8's
hands-on example uses exactly this.

## `panic-halt` and friends — someone still has to handle panics

```rust
use panic_halt as _;   // or panic-probe, panic-rtt-target, panic-semihosting, ...
```

`#![no_std]` still needs a `#[panic_handler]`, same requirement as
`hello-kernel`'s hand-written one
([Systems, Chapter 10](../systems/10-hello-kernel-uart-and-panics.md)).
The embedded ecosystem ships this as a pluggable crate instead — import
`panic_halt` and its `#[panic_handler]` (halt forever, the simplest
possible choice) is wired in for you; swap in `panic-probe` during
development to get the panic message streamed back over your debug
probe instead of silently halting with no diagnostic at all.
