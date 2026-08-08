# 9. `hello-kernel`: From QEMU's `-kernel` Flag to the First Rust Code

> Continues the [`hello-kernel` case study](./07-hello-kernel-overview.md)
> — imported from a separate project, not part of this repo.

## QEMU loads the ELF and jumps in

```
qemu-system-aarch64 -M virt -cpu cortex-a72 -nographic -kernel <elf>
```

(wired up as the cargo `runner` in `.cargo/config.toml` — chapter 8 —
so `cargo run` invokes this automatically.) `-M virt` emulates a
generic aarch64 "virt" board — RAM at `0x40000000`, a PL011 UART at
`0x0900_0000`, no real firmware/BIOS involved. With `-kernel`, QEMU
parses the ELF, copies its segments into guest RAM, and sets the CPU's
program counter to the ELF entry point — `_start` at `0x40080000`,
exactly what chapter 8's linker script recorded. `-nographic` routes
the emulated UART to your terminal's stdio instead of opening a
graphical window.

## Why isn't a real bootloader needed here?

It looks like a step real hardware always has got skipped — and it did,
but only because **QEMU's `-kernel` flag is standing in for the
bootloader**, not because this project's entry-point trick makes one
unnecessary in general.

On real hardware, at power-on the CPU doesn't start executing your
code — it starts executing whatever is wired to a fixed reset address,
which is ROM, not RAM. That first code exists because none of the
things this project takes for granted are true yet:

- **RAM usually isn't usable yet.** DRAM controllers need to be
  configured (timings, refresh rates — see
  [Chapter 3](./03-ram-and-virtual-memory.md) for what a memory
  controller actually does) before memory is even readable. You can't
  jump into RAM you haven't initialized.
- **The kernel image lives on storage, not memory.** It has to be read
  off an SD card, flash chip, or disk into RAM first — something has to
  know the storage hardware and filesystem/partition format
  ([Chapter 2](./02-disks-sectors-and-addressing.md)) to do that copy.
  `ENTRY(_start)` and `0x40080000` are meaningless until the bytes at
  that address actually exist in RAM.
- **Nothing has told an ELF loader "load this file" yet.** An ELF
  header's `e_entry` field is just a number sitting in a file; something
  has to parse that file, copy its loadable segments to the addresses
  they claim, and *then* jump to `e_entry`. That parsing/copying/jumping
  *is* the bootloader's job.

This is normally done in stages — e.g. on real aarch64 boards: a ROM
bootloader → an SPL → U-Boot or UEFI → the kernel — the same multi-stage
shape [Chapter 5](./05-boot-process-bios-uefi.md) describes for
BIOS/MBR and UEFI/GPT on more conventional hardware, each stage doing
just enough hardware bring-up to load and hand off to the next.

QEMU's `-kernel <elf>` flag collapses all of that away for development:
before the emulated CPU executes a single instruction, QEMU (running as
a normal host process, with full access to the file and to "guest RAM,"
which is just a chunk of host memory) parses the ELF itself, copies its
loadable segments into guest RAM, and only then starts the vCPU with PC
set to `e_entry`. In other words, **QEMU *is* the bootloader here** — it
does the loading in the emulator instead of in guest code. That's
exactly why this project has no boot stages: the one piece of software
that would normally do that work has been replaced by the `-kernel`
convenience feature, which exists only for development/testing and has
no equivalent on real hardware.

## `_start` — the first instruction that runs

```rust
#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn _start() -> ! {
    naked_asm!(
        "adrp x0, __stack_top",
        "add x0, x0, :lo12:__stack_top",
        "mov sp, x0",
        "bl kernel_main",
        "1:",
        "wfe",
        "b 1b",
    )
}
```

At this point there is **no stack** — the CPU reset with `sp`
undefined, so this function must be `naked` (raw assembly, no
Rust-generated prologue that would touch the stack) and `no_mangle` (so
the symbol name in the linker script matches literally).

- `adrp`/`add ... :lo12:` compute the address of `__stack_top` using
  PC-relative addressing (the standard aarch64 idiom for loading a
  link-time constant), loading it into `x0`.
- `mov sp, x0` establishes the stack — from this instruction on, normal
  Rust function calls (which push/pop the stack) are safe to make.
- `bl kernel_main` calls into ordinary Rust code.
- The `1: wfe / b 1b` pair is a fallback: if `kernel_main` ever returned
  (it can't — its return type is `!`), the core would sit in a
  low-power wait loop instead of executing whatever garbage follows in
  memory.

## `kernel_main` — the "kernel" itself

```rust
extern "C" fn kernel_main() -> ! {
    uart_puts("Hello, kernel!\n");
    loop {
        unsafe { asm!("wfe") };
    }
}
```

Prints the banner (chapter 10 covers exactly how), then parks the core
forever in `wfe` ("wait for event" — an aarch64 instruction that sleeps
the CPU until an interrupt or event wakes it, cheaper than spinning).
This is the entire kernel — no scheduler, no interrupts enabled, no
further work. Chapter 10 covers how the string actually reaches your
terminal, and what happens if something panics before getting this far.
