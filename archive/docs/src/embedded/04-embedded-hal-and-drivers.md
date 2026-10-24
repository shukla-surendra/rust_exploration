# 4. `embedded-hal` & Drivers: Chip-Agnostic Peripheral Traits

## The problem: every chip's GPIO/SPI/I2C API looks slightly different

[Chapter 2](./02-pac-hal-and-registers.md)'s HAL gives you
`led.set_high()` — friendly, but specific to `stm32f4xx-hal`'s own
types. A driver crate for, say, an accelerometer over I2C shouldn't
have to be rewritten for every chip family it might run on. The
`embedded-hal` crate solves this with a small set of shared traits every
chip's HAL implements:

```rust
pub trait OutputPin {
    type Error;
    fn set_high(&mut self) -> Result<(), Self::Error>;
    fn set_low(&mut self) -> Result<(), Self::Error>;
}
```

This is [Traits](../foundation/traits.md)'s core idea — "decouple what
code needs from which concrete type provides it" — applied to physical
pins instead of software behavior. A driver crate writes against
`OutputPin`/`InputPin`/`SpiDevice`/`I2c` and similar traits; any chip's
HAL that implements them plugs in, unmodified:

```rust
fn blink<P: OutputPin>(pin: &mut P) {
    pin.set_high().ok();
    pin.set_low().ok();
}
```

`blink` never mentions STM32, RP2040, or any specific chip — the same
generic-over-a-trait-bound pattern from
[Traits & Generics](../workbook/04-traits-and-generics.md), here letting
one function compile against dozens of unrelated chip families.

## Driver crates: the actual payoff

```toml
[dependencies]
bme280 = "0.5"        # temperature/humidity/pressure sensor, over I2C
mpu6050 = "0.1"        # accelerometer/gyroscope, over I2C
```

```rust
let mut sensor = bme280::BME280::new(i2c, 0x76);
sensor.init(&mut delay).unwrap();
let measurements = sensor.measure(&mut delay).unwrap();
```

`i2c` here is anything implementing `embedded-hal`'s `I2c` trait —
whichever HAL your specific board's chip provides. This is the entire
reason the trait layer exists: hundreds of published driver crates for
real-world sensors, displays, and radios, each written once against
`embedded-hal`, usable on any of dozens of supported chip families
without the driver author needing to know about — or test on — your
specific board.

## `embedded-hal` 1.0's fallible-by-default design

Every trait method above returns a `Result`, even `set_high()` — unlike
a typical OS's GPIO write, an embedded peripheral operation can
genuinely fail (an I2C bus can NACK, an SPI transaction can time out),
and there's no OS underneath to translate that into an exception the
way [Error Handling](../foundation/error-handling.md) or
[Option, Result & unwrap_or_else](../foundation/option-result.md)'s
`?`-propagation story assumes for a hosted program. The same discipline
applies here, just with hardware faults instead of file-not-found as
the routine failure mode being modeled.

## Comparing back to `hello-kernel` and OS-level drivers

[OxideOS's driver docs](../oxideos/study/07_drivers.md) describe three
driver "models" (port-mapped, interrupt-driven, PCI) for a *hosted OS
kernel* talking to devices *it* discovers and owns exclusively.
`embedded-hal` solves a related but distinct problem: **portability
across chips**, for code that doesn't have an OS underneath it managing
device ownership at all — your embedded program *is* the only thing
that will ever touch that I2C bus, so there's no equivalent of a VFS or
device-file abstraction needed, just a shared trait vocabulary so
driver code isn't duplicated per chip.
