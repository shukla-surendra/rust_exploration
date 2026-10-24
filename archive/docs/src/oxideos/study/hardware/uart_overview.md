# UART — The Protocol, Full Reference

**Start here for UART specifically.** This is the protocol-level
overview — what it is, why it's shaped the way it is, and everywhere it
shows up. [`16550_uart.md`](16550_uart.md) and
[`modern_serial_debug.md`](modern_serial_debug.md) are its register-level
companions, covering the exact chip OxideOS drives (16550, x86 PMIO)
and what replaced/paralleled it (PL011 on ARM, USB-to-UART bridges, BMC
Serial-over-LAN) — this doc is the concept everything in both of those
is an instance of.

## 1. What UART actually does

UART — **U**niversal **A**synchronous **R**eceiver/**T**ransmitter — is
a chip (or, more often today, a small block of logic inside a larger
chip) that converts data your CPU works with in parallel, all-at-once
form, into a single-wire serial stream of bits, and back again:

```
Device A                         Device B
--------                         --------
TX  -------------------------->  RX
RX  <--------------------------  TX
GND ---------------------------- GND
```

Three wires, minimum: **TX** (transmit — data going out), **RX**
(receive — data coming in), and a shared **GND** (ground — without a
common reference voltage, "high" and "low" on the signal wires aren't
even meaningfully defined between the two devices). Note that TX on one
side connects to RX on the other, not TX-to-TX — a detail that trips up
almost everyone the first time they wire two boards together by hand.

**At the byte level**, sending the character `'A'` (`0x41`) looks like
the outline you started with:

```
CPU
 |
 |  0x41
 v
UART
 |
 |  serial bits
 v
TX ──────> RX
             |
             v
           UART
             |
             v
            CPU
```

**At the bit level** — the part worth actually seeing once, because it's
where "asynchronous" (section 2) becomes concrete instead of just a
word — a UART never sends a raw byte. It wraps each byte in a **frame**:
one **start bit** (always 0, so the line dropping from its idle-high
state is itself the signal "a byte begins now"), the data bits
themselves (usually 8, least-significant-bit first — a genuine, easy-
to-get-backwards detail), an optional parity bit, and one or more
**stop bits** (always 1, returning the line to idle and guaranteeing a
detectable gap before the next start bit). `0x41` = binary `01000001`,
transmitted LSB-first, framed as **8N1** (8 data bits, no parity, 1 stop
bit — the section-2 notation, now with a concrete byte behind it):

```
idle  start  d0 d1 d2 d3 d4 d5 d6 d7  stop  idle
 1      0     1  0  0  0  0  0  1  0    1    1
      <-------------- one bit time each --------------->
```

(`0x41` = `0100 0001` written the normal, MSB-first way. LSB-first
transmission reads that same byte back-to-front: bit 0 — the rightmost
`1` — goes out first (`d0=1`), then bits 1 through 5 (all `0`) go out
as `d1..d5=0`, then bit 6 — the `1` in `0100` — goes out as `d6=1`, and
finally bit 7 (`0`, the leftmost digit) goes out last as `d7=0`. Trace
it against the binary yourself once — it's the fastest way to make
LSB-first stop being an abstract phrase.)

## 2. Why is it called "asynchronous"?

**There is no separate clock wire.** Compare directly against SPI, a
common *synchronous* serial protocol:

```
SPI:                              UART:
MOSI ─────────>                   TX ─────────>
MISO <─────────                   RX <─────────
SCLK ─────────>   ← clock         GND ─────────
CS   ─────────>
```

SPI's `SCLK` line ticks on every single bit — the receiver doesn't need
to know the data rate in advance at all; it just samples whenever the
clock edge says to. UART has no such line, which means **both sides
have to already agree**, before a single byte is sent, on:

```
Baud rate: 115200
Data bits: 8
Parity:    None
Stop bits: 1
```

— conventionally written `115200 8N1`. If one side is configured for
9600 baud and the other for 115200, both sides will run their UART
hardware happily, produce a stream of bits, and receive complete
garbage — there's no error, no negotiation, just two devices
confidently disagreeing about how long a "bit" lasts.

**Why this actually works without a clock wire, and where the limit is**:
the receiver doesn't need continuous clock ticks — it only needs to know
*roughly* when to sample each bit, and the start bit's falling edge
gives it that reference point once per byte. From there, it counts bit
periods at its own locally-configured rate and samples near the middle
of each one. This is why UART frames stay short (a start bit, up to 8-9
data/parity bits, a stop bit — rarely more than 10-12 bits total): the
receiver's clock and the transmitter's clock are two physically separate
oscillators that are never perfectly identical, so any small mismatch
between them **accumulates** bit over bit within a single frame. At
115200 baud each bit lasts about 8.68 microseconds; over a 10-bit frame,
even a small percentage clock mismatch can drift the sampling point
enough to misread the last bit or two. The start bit resets that drift
to zero at the beginning of every single frame — which is the actual
engineering reason UART frames stay short instead of streaming
arbitrarily long runs of bits with no re-sync point.

## 3. What devices support UART?

### Microcontrollers

Close to universal: **ESP32, STM32, Arduino/AVR, RP2040 (Raspberry Pi
Pico), NXP, PIC, TI** microcontrollers all ship dedicated hardware UART
peripherals. A typical debug/programming setup:

```
ESP32                         Computer
------                        --------
TX   ----------------------> RX
RX   <----------------------  TX
GND  ----------------------- GND
```

used constantly for flashing firmware, printing debug logs, and sending
ad-hoc commands during development.

### CPUs and computers — where this section connects to OS development

This is the same picture, one level more general:

```
CPU
 |
 | MMIO registers
 v
UART controller
 |
 | TX/RX pins
 v
Serial cable
```

and it's not hypothetical in this workspace — it's the **exact
mechanism** behind two real, working projects already built and
verified here:

- **x86-64**: a 16550-compatible UART, reached via **port-mapped I/O**
  (`in`/`out` to ports `0x3F8`-`0x3FD`) — full register table in
  [`16550_uart.md`](16550_uart.md), real driver source at
  `kernel/src/kernel/drivers/serial.rs`. A hand-written, from-scratch,
  bare-metal version of the exact same init/poll/transmit sequence is
  in `rust_exploration/asm_examples/12_bare_metal_uart/x86_64_multiboot.s`
  — boots directly in QEMU with no OS at all and really prints through
  real port I/O.
- **ARM64**: a PL011 UART, reached via **memory-mapped I/O** (ordinary
  loads/stores to a fixed address, `0x09000000` on QEMU's `virt`
  machine) — covered in [`modern_serial_debug.md`](modern_serial_debug.md),
  real driver source at `arch/aarch64/serial.rs`. The real,
  independently-working `hello-kernel` project
  (`~/projects/hello-kernel/src/main.rs`) does exactly this — poll the
  Flag Register, write the Data Register — in about ten lines of Rust
  around two `core::ptr::read_volatile`/`write_volatile` calls. A
  hand-written assembly version, `arm64_qemu_virt.s` in this same
  `asm_examples/12_bare_metal_uart/` folder, does the identical thing
  in raw `ldr`/`str` and is where a real, instructive bare-metal bug
  got caught and fixed (see section 7).

Both are exactly the general mechanism [`asm/11-io-ports-and-mmio.md`](../../asm/11-io-ports-and-mmio.md)
and its plain-language companion
[`systems/11-talking-to-hardware-pmio-vs-mmio.md`](../../systems/11-talking-to-hardware-pmio-vs-mmio.md)
cover — UART is simply the running example both of those chapters use,
because it's the smallest real device that makes the whole CPU-to-
hardware communication story concrete.

### Sensors and modules

A huge, genuinely common category communicates over UART specifically
because it's dead simple to implement on both ends and needs no shared
clock line to route: **GPS/GNSS modules, Bluetooth modules, GSM/LTE
modules, Wi-Fi modules, some fingerprint sensors, RFID modules, motor
controllers, industrial equipment, debug interfaces.**

```
GPS module
    |
    | UART
    v
ESP32
```

A GPS module, once powered, just continuously streams sentences with no
request needed:

```
$GPGGA,...
$GPRMC,...
```

(NMEA 0183 format — each line is a self-contained, comma-separated
sentence; a receiver just reads bytes until it sees a newline and parses
whatever arrived, no request/response handshake at all.)

## 4. UART vs. USB

**A USB port is not necessarily UART**, and conflating the two is a
common source of confusion the first time you plug a dev board into a
laptop that has no UART header at all (most modern laptops, including
Apple Silicon Macs — [`modern_serial_debug.md`](modern_serial_debug.md#what-actually-happened-to-the-uart)
covers exactly what replaced physical UART ports on consumer hardware).
What's actually happening:

```
Computer USB
     |
     v
USB-to-UART chip
     |
     v
UART TX/RX
     |
     v
ESP32
```

Common **USB-to-UART bridge chips**: **FTDI** (the FT232 family),
**CP210x** (Silicon Labs), **CH340** (WCH) — when an ESP32 dev board
shows up as `/dev/ttyUSB0` (Linux) or `/dev/tty.usbserial-*` (macOS), one
of these chips is sitting on the board, translating real USB packets
into the exact same TX/RX/GND electrical signaling described in section
1. [`modern_serial_debug.md`](modern_serial_debug.md#what-actually-happened-to-the-uart)
covers the deeper detail worth knowing once this clicks: these bridge
chips don't invent a new interface — they present themselves to the OS
as a **virtual 16550-compatible device**, meaning software written
against the register semantics in [`16550_uart.md`](16550_uart.md)
largely still applies conceptually even when there's no physical 16550
silicon anywhere in the chain.

## 5. UART vs. USART

Microcontroller datasheets constantly say **USART** instead of UART —
**U**niversal **S**ynchronous/**A**synchronous **R**eceiver/**T**ransmitter.
A USART is a strict superset: it can run in UART's asynchronous,
no-clock-wire mode (everything above), *or* switch into a synchronous
mode that adds a clock line back in (closer to SPI's shape) for
applications that need the higher, drift-free data rates a shared clock
allows. Most embedded UART peripherals people casually call "the UART"
are, strictly, USART peripherals being used in their asynchronous mode
— the terms get used interchangeably in practice because asynchronous
mode is what's almost always meant.

## 6. The mental model for OS/kernel development

```
             SOFTWARE
                │
                │ read/write registers
                ▼
        ┌─────────────────┐
        │ UART Controller │
        └─────────────────┘
             │       │
            TX       RX
             │       │
             ▼       ▲
        ────────────────
          physical wire
        ────────────────
```

The CPU never "sends a voltage" directly. Every layer in between is
real and matters:

```
Rust/C code
    ↓
write UART register        ← in/out (x86 PMIO) or a plain store (ARM64 MMIO)
    ↓
UART hardware
    ↓
TX pin
    ↓
electrical signal
    ↓
RX pin of another device
    ↓
other UART hardware
    ↓
CPU reads UART register
```

This is exactly why UART is such a good concept to learn OS development
through — it's the smallest real example of the general pattern
**CPU → memory-mapped (or port-mapped) hardware registers → peripheral
→ physical signal**, with nothing else in the way. No filesystem, no
scheduler, no interrupt controller required to understand it end to end
(though a real driver typically adds an interrupt instead of pure
polling once it's doing more than printing debug text — [Chapter
4](../../asm/04-interrupts-and-exceptions.md) covers that layer, and
[`asm/11`](../../asm/11-io-ports-and-mmio.md#polling-vs-interrupts-the-two-ways-to-know-a-device-is-ready)
names exactly where polling stops being good enough).

## 7. Everywhere UART actually shows up in OS development

This is the part worth being explicit about, since it's easy to think of
UART as "just a debug feature" rather than the load-bearing piece of
infrastructure it actually is at boot:

**It's almost always the very first driver initialized, before
anything else — including the memory allocator.** Both real kernel
projects in this workspace do this: OxideOS's `serial.rs` and
`hello-kernel`'s `_start` (`~/projects/hello-kernel/src/main.rs`) bring
up UART before any heap/allocator exists, specifically *so that* every
later step — memory setup, interrupt setup, everything that can
plausibly fail — has somewhere to report failure to. A kernel with no
working allocator yet can still call `uart_putc` in a loop; it usually
can't do much else observable at all.

**Panic and early-boot output has nowhere else to go.** Look at
`hello-kernel`'s actual `panic_handler` — it calls `uart_puts("panic!\n")`
and halts. Before a framebuffer/GUI driver exists, before a filesystem
exists to write a log file to, UART is *the only channel available* to
say anything went wrong at all.

**The embedded Linux serial console** is this same mechanism, scaled up
to a full OS: a bootloader (U-Boot, GRUB) prints its own progress over
UART, hands off to the kernel, which keeps writing to the same UART
throughout boot, arriving at a login prompt over the wire:

```
Bootloader
    ↓
Kernel
    ↓
init/system
    ↓
UART
    ↓
Your terminal
```

producing exactly the familiar
```
Booting Linux...
Initializing memory...
Starting kernel...
login:
```
you see connecting a serial cable (or, in 2026, an SSH-over-serial-
concentrator, or a cloud provider's "serial console" web feature) to
almost any embedded board or headless server.

**Kernel debuggers** (`kgdb` and equivalents) run their entire
protocol — breakpoints, memory inspection, single-stepping — over a
UART connection to a second machine, precisely because it works even
when the system under test is in a state too broken for anything
network-stack-dependent to function.

**In production server hardware**, this exact mechanism is still there,
just relocated: [`modern_serial_debug.md`](modern_serial_debug.md#what-actually-happened-to-the-uart)
covers how a server's BMC (Baseboard Management Controller — iLO on HPE,
iDRAC on Dell) terminates a real UART locally and tunnels it over the
network as **Serial-over-LAN**, so a remote engineer gets the identical
"talk to the machine before its OS has even finished booting" access a
physical serial cable would give — the server OS's own code is still
writing to ordinary UART registers with no idea the bytes are leaving
over Ethernet instead of a wire.

**A real, worked bug from this exact codebase**: the bare-metal ARM64
example in `asm_examples/12_bare_metal_uart/arm64_qemu_virt.s` hung on
first boot, and the instinctive diagnosis — "the UART's control
register was never configured, so its status register is lying" —
turned out to be *wrong*, confirmed by comparing against `hello-kernel`
(which polls the identical PL011 registers with **no** control-register
init at all, and works). The actual bug was an uninitialized stack
pointer corrupting state on the first function call, nothing to do with
the UART's configuration at all. Full trace-by-trace writeup in
[`asm/11-io-ports-and-mmio.md`](../../asm/11-io-ports-and-mmio.md#the-real-bug--and-the-wrong-diagnosis-that-delayed-finding-it)
— worth reading once the mental model above feels solid, as a concrete
example of how a plausible-sounding hardware explanation can still be
the wrong one.
