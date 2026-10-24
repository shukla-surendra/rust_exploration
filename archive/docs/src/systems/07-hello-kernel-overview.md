# 7. Case Study: `hello-kernel` — a Real Bare-Metal Rust Kernel

> Imported from a separate project (`hello-kernel`, not part of this
> repo). Chapters 7–10 walk through it end to end — everything chapters
> 2 and 5 described in the abstract (boot sectors, "who loads the
> kernel," entry points) here as a real, running ~50-line Rust program
> you can build and boot in QEMU yourself.

## What it is

A minimal **freestanding** (`#![no_std]`, `#![no_main]`) Rust binary
that boots as a bare-metal aarch64 "kernel" under QEMU and prints
`Hello, kernel!` over a PL011 UART — no bootloader crate involved. QEMU's
`virt` machine loads the compiled ELF directly via `-kernel` and jumps
straight to `_start`, the same way real firmware would jump into a real
kernel, just with QEMU itself standing in for every earlier boot stage
(chapter 9 covers exactly what that means and what's being skipped).

`#![no_std]` means no Rust standard library is linked in at all — there
is no OS underneath this program to provide one. `#![no_main]` means no
C runtime startup code is inserted before your code runs either — there
is no `main` function in the usual sense, just whatever code is placed
at the address the CPU's program counter gets set to. Both attributes
together are what make this a **freestanding binary**: a program with
no assumptions about an environment being ready to receive it, which is
the actual definition of "bare metal."

## Requirements

- `rustup target add aarch64-unknown-none` — a target with no OS, no
  libc, nothing but the raw aarch64 CPU ABI (see chapter 8 for what the
  target and linker wiring actually do)
- `qemu-system-aarch64` (`brew install qemu`)

## Running it

```sh
cargo run --release
```

`.cargo/config.toml` wires `cargo run` to invoke `qemu-system-aarch64
-M virt -cpu cortex-a72 -nographic -kernel <binary>` automatically —
chapter 9 covers exactly what each of those flags does. The kernel
prints its message and then halts in a low-power wait loop; quit QEMU
with `Ctrl-A X`, or from another shell with `pkill qemu-system-aarch64`.

## How it works, in one screen

The full walkthrough is chapters 8–10; the shape of it:

- `linker.ld` places `_start` at `0x40080000` (inside the `virt`
  machine's RAM, which starts at `0x40000000`) and reserves a 64 KiB
  stack above the compiled image.
- `_start` (a `naked` function — no compiler-generated prologue, because
  there is no stack yet for a prologue to use) sets `sp` to the top of
  that reserved stack and calls `kernel_main`.
- `kernel_main` polls the PL011 UART at the `virt` machine's fixed
  address `0x09000000` and writes bytes to its data register directly.
- On an aarch64 host (e.g. Apple Silicon), QEMU can use hardware
  acceleration (`-accel hvf`) since guest and host architectures match —
  not needed for a program this small, but relevant the moment the
  kernel does real, sustained work.

## Prerequisites from earlier in this section

If any of the following feel shaky, the referenced chapter covers it
before this case study assumes it:

- **Addresses, bytes, volatile memory** — [Chapter 1](./01-prerequisites-bits-bytes-and-addressing.md),
  used constantly here (`0x40080000`, `0x09000000`, and the UART's
  `read_volatile`/`write_volatile` accesses in chapter 10).
- **Who normally loads a kernel, and how boot addressing works on real
  hardware** — [Chapter 5](./05-boot-process-bios-uefi.md)'s BIOS/UEFI
  boot chain is the real-hardware version of what chapter 9's "why isn't
  a bootloader needed" section explains QEMU is standing in for here.

Chapter 8 picks up with the build itself — the target, the linker
script, and a specific Rust-edition detail this project's source
already relies on.
