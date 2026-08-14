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

## A third bootstrapping path neither project here uses

The two case studies above cover two real approaches: `hello-kernel`
gets loaded directly by QEMU's `-kernel` flag (no bootloader at all —
[Systems, Chapter 9](../systems/09-hello-kernel-boot-to-execution.md)),
and OxideOS is loaded by **Limine**, a modern bootloader using its own
request/response protocol
([OxideOS Concepts, Chapter 1](../oxideos/oxide_cocepts/01_boot_process.md)).
A third path is common enough in Rust OS-dev tutorials to be worth
naming even though it's not used here: the **`bootloader` crate +
`bootimage`** combination (the approach the well-known "Writing an OS
in Rust" blog series uses).

**A custom target spec instead of a built-in triple.** `hello-kernel`
([Systems, Chapter 8](../systems/08-hello-kernel-build-and-linking.md))
uses `aarch64-unknown-none`, a target triple `rustc` already ships. x86
bare-metal projects following the `bootloader`/`bootimage` path instead
write their own target as JSON — there's no built-in "no bootloader, no
libc, raw x86_64" triple:

```json
{
  "llvm-target": "x86_64-unknown-none",
  "arch": "x86_64",
  "os": "none",
  "executables": true,
  "linker-flavor": "ld.lld",
  "panic-strategy": "abort",
  "disable-redzone": true,
  "features": "-mmx,-sse,+soft-float",
  "relocation-model": "static"
}
```

Building against a custom target means `core`/`alloc` don't exist
precompiled for it — they have to be built from source alongside your
own code, which is what the unstable `-Z build-std=core,alloc` flag
(and a **nightly** toolchain — see
[Cargo, Modules, Testing & Macros](../workbook/10-cargo-modules-testing-macros.md))
is for.

**`bootimage` — a linking step neither case study here needs.** Plain
`cargo build` against a target like the one above produces a `.elf` —
not directly bootable by BIOS. The `bootimage` crate (`cargo install
bootimage`) combines that ELF with the `bootloader` crate (a real,
from-scratch Stage 1 + Stage 2 BIOS bootloader, written in Rust,
implementing exactly the
[Stage 1/Stage 2 split](../systems/05-boot-process-bios-uefi.md)
described in Systems chapter 5) into one flat `.bin` image BIOS can
actually load. `cargo bootimage && qemu-system-x86_64 -drive
format=raw,file=<image>.bin` then runs it — no `-kernel` flag, because
there's a real bootloader in the loop this time, unlike `hello-kernel`.

**The VGA text buffer — the classic alternative to UART output.** Where
`hello-kernel`
([Systems, Chapter 10](../systems/10-hello-kernel-uart-and-panics.md))
writes bytes to a UART register, BIOS-era x86 has a second option with
no wiring required at all: text written to physical address `0xB8000`
appears directly on screen, 2 bytes per character (1 ASCII byte + 1
color-attribute byte), in an 80×25 grid the BIOS already initialized
before handing off control:

```rust
let vga_buffer = 0xb8000 as *mut u8;
let msg = b"Hello World from Rust OS!";
for (i, &b) in msg.iter().enumerate() {
    unsafe {
        *vga_buffer.add(i * 2) = b;           // ASCII byte
        *vga_buffer.add(i * 2 + 1) = 0x0F;    // white on black
    }
}
```

OxideOS deliberately skips this in favor of a real linear framebuffer —
see [Graphics & GUI](../oxideos/oxide_cocepts/05_graphics_and_gui.md)
for why pixel-level graphics needs a different approach than 80×25 text
cells.

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
