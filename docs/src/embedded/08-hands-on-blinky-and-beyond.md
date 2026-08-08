# 8. Hands-On: Blinky and Beyond

Embedded Rust's "hello world" is blinking an LED — this walks through a
complete, minimal project the same way
[Systems, Chapter 6](../systems/06-hands-on-building-a-disk-image-in-rust.md)
and the [`hello-kernel` case study](../systems/07-hello-kernel-overview.md)
did for OS-level Rust, using every piece from chapters 1–5.

## The project

```toml
# Cargo.toml
[package]
name = "blinky"
edition = "2021"

[dependencies]
cortex-m = "0.7"
cortex-m-rt = "0.7"
panic-halt = "0.2"
stm32f4xx-hal = { version = "0.20", features = ["stm32f411"] }
```

```
/* memory.x — Chapter 1 */
MEMORY
{
    FLASH : ORIGIN = 0x08000000, LENGTH = 512K
    RAM   : ORIGIN = 0x20000000, LENGTH = 128K
}
```

```rust
// src/main.rs
#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;                              // Chapter 1
use stm32f4xx_hal::{pac, prelude::*};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();     // Chapter 2 — PAC
    let gpioa = dp.GPIOA.split();                    // Chapter 2 — HAL
    let mut led = gpioa.pa5.into_push_pull_output();

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();
    let mut delay = dp.TIM2.delay_ms(&clocks);

    loop {
        led.set_high();
        delay.delay_ms(500u32);
        led.set_low();
        delay.delay_ms(500u32);
    }
}
```

Every line traces to an earlier chapter: `#![no_std]`/`#![no_main]` and
`#[entry]` from [Chapter 1](./01-no-std-and-the-embedded-toolchain.md);
`pac::Peripherals::take()` and `gpioa.pa5.into_push_pull_output()` from
[Chapter 2](./02-pac-hal-and-registers.md); `panic_halt` from
[Chapter 1](./01-no-std-and-the-embedded-toolchain.md)'s panic-handler
discussion. No `asm!`, no manual register addresses, no hand-written
linker script beyond the two-number `memory.x` — this is the entire
point of the layered ecosystem from
[the overview](./00-overview.md): `hello-kernel` wrote every one of
these pieces by hand for a teaching example; here the ecosystem
supplies all of it, and your code is the four lines that actually
matter (get the LED pin, toggle it, wait).

## Running it — two ways

**On real hardware** (an STM32F411 Nucleo/Discovery board or similar,
per [Chapter 1](./01-no-std-and-the-embedded-toolchain.md)):

```sh
cargo install probe-rs-tools
cargo run --release   # flashes + runs via probe-rs, per .cargo/config.toml
```

**Under QEMU**, no hardware required — swap the target to a
QEMU-emulated Cortex-M machine (e.g. `lm3s6965evb`, a simpler chip QEMU
supports well) and a matching runner:

```toml
# .cargo/config.toml
[target.thumbv7em-none-eabihf]
runner = "qemu-system-arm -cpu cortex-m4 -machine lm3s6965evb -nographic -kernel"
```

```sh
cargo run --release
```

The exact same `-nographic -kernel <binary>` shape as
[`hello-kernel`'s own QEMU invocation](../systems/09-hello-kernel-boot-to-execution.md)
— QEMU parses the compiled ELF and boots it directly, no separate
bootloader step, the identical mechanism
[Systems, Chapter 9](../systems/09-hello-kernel-boot-to-execution.md)
explained in depth for aarch64. (LED output isn't visible under QEMU
without a model of the specific board's GPIO-to-display wiring — swap
the LED toggle for `cortex_m_semihosting::hprintln!` output to see
something in your terminal instead, when running this way.)

## Extending it — natural next steps, tied to the rest of this section

- **Read a button, debounced** — `into_pull_up_input()` +
  `embedded-hal`'s `InputPin` ([Chapter 4](./04-embedded-hal-and-drivers.md)).
- **Blink via a timer interrupt instead of a busy-wait `delay`** —
  [Chapter 3](./03-interrupts-on-cortex-m.md)'s `#[interrupt]`, toggling
  the LED from `TIM2`'s handler instead of blocking `main` on `delay_ms`.
- **Rewrite it with Embassy** — [Chapter 6](./06-async-embedded-with-embassy.md)'s
  `Timer::after_millis(500).await` in place of the blocking `delay_ms`
  call, and a second concurrent task (e.g. watching a button) that a
  blocking `loop` couldn't run alongside the blink at all.
- **Add a real sensor driver** — any `embedded-hal`-based crate
  ([Chapter 4](./04-embedded-hal-and-drivers.md)) over the same
  project's I2C peripheral.
- **Check your worst-case stack usage** — `cargo-call-stack`, from
  [Chapter 5](./05-memory-constraints-and-heapless.md)'s stack-overflow
  warning, once the project has grown past this minimal example.
