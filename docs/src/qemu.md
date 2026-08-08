# QEMU: Why It Exists, and What It's Actually For

QEMU has already been running underneath nearly every hands-on example
in this book — [`hello-kernel`](./systems/07-hello-kernel-overview.md),
[OxideOS](./oxideos/00-overview.md), and
[the embedded blinky walkthrough](./embedded/08-hands-on-blinky-and-beyond.md)
all boot through it — without it ever being explained on its own terms.
This page is that explanation: what it actually is, how it works, and
the concrete, real-world reasons people reach for it beyond "the tool
this book happens to use."

## What QEMU actually is — two different tools under one name

- **System emulation** (`qemu-system-x86_64`, `qemu-system-aarch64`,
  `qemu-system-arm`, ...) — emulates an **entire machine**: a CPU, RAM,
  a disk controller, a UART, an interrupt controller, sometimes a GPU.
  This is everything used throughout this book —
  [`hello-kernel`'s `-M virt`](./systems/09-hello-kernel-boot-to-execution.md)
  emulates a whole generic aarch64 board.
- **User-mode emulation** (`qemu-x86_64`, `qemu-aarch64`, ...) — runs a
  **single foreign-architecture binary** directly, translating its
  instructions on the fly while passing every syscall straight through
  to the *host* kernel — no virtual machine, no emulated disk or
  bootloader, just "run this one ARM binary on my x86 machine as if it
  were native." This is what powers Docker's multi-architecture builds
  (`docker buildx`) under the hood — a real, widely-used production use
  case most people don't realize is QEMU.

Everything in this book uses system emulation; user-mode emulation is
worth knowing exists because it's the answer to a related but different
question ("run one foreign binary" vs. "boot a whole foreign OS").

## How it works: software translation vs. hardware acceleration

- **TCG (Tiny Code Generator)** — QEMU's default, software-only mode:
  it translates guest instructions into host instructions dynamically,
  block by block, as the guest runs. This works for **any** combination
  of host and guest architecture — it's exactly why
  [`hello-kernel`'s aarch64 kernel](./systems/07-hello-kernel-overview.md)
  can be built and tested on an x86-64 CI runner, or why an
  [embedded Cortex-M target](./embedded/08-hands-on-blinky-and-beyond.md)
  can be tested without owning that silicon at all — TCG doesn't care
  that the host CPU can't natively execute the guest's instructions.
- **Hardware-accelerated virtualization** — when the host and guest
  architecture *match*, QEMU can instead let the host CPU execute guest
  instructions almost directly, via a hypervisor interface: **KVM** on
  Linux, **HVF** (Hypervisor.framework) on macOS, **WHPX** on Windows.
  This is dramatically faster than TCG — [`hello-kernel`'s own
  README](./systems/07-hello-kernel-overview.md) notes exactly this
  tradeoff (Apple Silicon running an aarch64 guest can use `-accel hvf`,
  though it's unnecessary for a program that small), and OxideOS ships
  a dedicated `make run-kvm-x86_64` target for the same reason on
  matching-architecture Linux hosts.

The practical rule: **cross-architecture testing needs TCG** (there's no
alternative); **same-architecture testing benefits enormously from
hardware acceleration**, and QEMU picks TCG by default, accelerated
modes opt-in via `-accel`.

## Real use case 1: OS/kernel development without real hardware

The use case every earlier chapter in this book has already been living
inside. Compare booting a kernel on real hardware to booting it in
QEMU:

| | Real hardware | QEMU |
|---|---|---|
| Reboot cycle | Seconds to minutes, physical access needed | Instant — `cargo run` again |
| Risk of a bad build | Can genuinely brick a device | Zero — it's a process you can just kill |
| Console output | Needs a serial cable or a screen | `-nographic`/`-serial stdio` pipes it straight to your terminal |
| Cost of trying 50 variations in an hour | Impractical | Completely normal |

This is precisely why [`hello-kernel`](./systems/07-hello-kernel-overview.md)
and OxideOS both develop exclusively (or primarily) against QEMU, only
touching real hardware once something already works.

## Real use case 2: cross-architecture development and testing

TCG (above) means you can develop and test **aarch64 kernel code on an
x86-64 laptop**, or **x86-64 kernel code on Apple Silicon** — exactly
what lets this book cover both
[`hello-kernel` (aarch64)](./systems/07-hello-kernel-overview.md) and
[OxideOS (x86-64)](./oxideos/00-overview.md) as reference material
regardless of which machine you're actually reading it on. The same
applies to [the embedded section](./embedded/08-hands-on-blinky-and-beyond.md)'s
Cortex-M example — QEMU's `lm3s6965evb`/`mps2-an385` machine types let
you test real microcontroller-target code without owning that specific
board.

## Real use case 3: CI/CD pipelines

Booting a kernel or embedded target as part of an automated test run
(see [CI/CD](./production/07-ci-cd.md)) needs something that runs
headless, scriptable, on a shared runner with no physical hardware
attached at all. `qemu-system-* -nographic -kernel <binary>` is exactly
that — no GUI, no operator, deterministic exit behavior a CI script can
check. This is also, again, exactly what `docker buildx`'s multi-arch
builds lean on QEMU's user-mode emulation for in real production CI
pipelines industry-wide, not just OS-dev.

## Real use case 4: interactive kernel debugging via GDB — arguably QEMU's killer feature for this exact kind of work

Something not yet mentioned anywhere else in this book, and one of the
single best reasons to prefer QEMU over real hardware for kernel work:
**QEMU can pause a guest at boot and expose a GDB server**, letting you
attach a real debugger to code that hasn't even set up a stack yet —
something that would otherwise require a physical JTAG probe costing
real money.

```sh
qemu-system-aarch64 -M virt -cpu cortex-a72 -nographic -kernel target/.../hello-kernel -s -S
```

`-S` freezes the CPU immediately at startup instead of running;
`-s` is shorthand for `-gdb tcp::1234`, starting a GDB server. In
another terminal:

```sh
gdb target/.../hello-kernel
(gdb) target remote :1234
(gdb) break kernel_main
(gdb) continue
(gdb) info registers
(gdb) stepi
```

You can single-step [`_start`'s hand-written `naked_asm!`](./systems/09-hello-kernel-boot-to-execution.md)
one instruction at a time, break at a specific physical address before
`kernel_main` even runs, inspect every register right after the `mov
sp, x0` that establishes the stack — this is genuinely how a lot of
real kernel-development debugging happens, and it works identically for
[OxideOS's](./oxideos/00-overview.md) far more complex boot sequence.

## Real use case 5: bootloader and firmware development

OxideOS ships both a real bootloader (`limine/`) and UEFI firmware
files (`ovmf/ovmf-code-x86_64.fd`, `ovmf/ovmf-vars-x86_64.fd`) alongside
its QEMU run targets — testing a bootloader or a UEFI firmware stack
*before* ever touching real hardware is a genuine, common use case
outside this book too (U-Boot's own upstream test suite leans on QEMU
heavily for exactly this reason).

## Real use case 6: full alternate-OS virtualization and disposable sandboxes

The "mainstream" QEMU use case most people already know it for, even if
they've never typed a `qemu-system-*` command by hand: running a
complete Linux/BSD/Windows VM (often via a friendlier front-end like
`virt-manager`/`libvirt`, both built on top of QEMU) — for testing a
specific distro, isolating untrusted or suspicious code in a disposable,
snapshot-able environment, or simply running a different OS locally
without dual-booting.

## Why QEMU specifically, vs. the alternatives

| | QEMU | Bochs | VirtualBox/VMware | Real hardware |
|---|---|---|---|---|
| Architecture coverage | x86, ARM, RISC-V, MIPS, PowerPC, SPARC, ... | x86 only | Host architecture only (mostly) | One, whatever you bought |
| Cross-arch testing | Yes (TCG) | No | No | No |
| Scriptable/headless | Excellent — exactly the `cargo run` wiring used throughout this book | Good, less common in this niche | Weaker, GUI-first tooling | N/A |
| Hardware acceleration | Yes (KVM/HVF/WHPX) when arch matches | No — always software-emulated, much slower | Yes, own hypervisor | Native, obviously |
| Built-in GDB stub | Yes, as shown above | Yes, and historically very precise for low-level x86 debugging | No, not designed for this | Needs a physical JTAG probe |
| Cost | Free, open source | Free, open source | Free tier / commercial | The hardware itself |

Bochs is worth naming specifically: it's a *pure* x86 software emulator
with historically very precise, very slow, instruction-by-instruction
fidelity — some OS-dev communities still prefer it for the very lowest-
level x86 debugging specifically because of that precision, but it has
none of QEMU's architecture breadth, acceleration options, or general
ecosystem momentum, which is why QEMU is the default choice used
throughout this book instead.

## The one real caveat: not every target has QEMU support

Directly parallel to [the Rust-toolchain caveat already documented for
Xtensa-based ESP32 chips](./embedded/00-overview.md#this-section-assumes-arm-cortex-m-specifically--heres-why) —
upstream QEMU doesn't support the Xtensa architecture either. Espressif
maintains their own QEMU fork for it, the same pattern as their forked
Rust compiler: when a chip uses an unusual, non-mainstream instruction
set, expect *both* the compiler toolchain and the emulator to require a
vendor fork rather than the stock, upstream tool. Mainstream
architectures (x86-64, aarch64, RISC-V, and Cortex-M specifically) all
have full upstream QEMU support — the target used everywhere else in
this book.

## Flag cheat sheet — every flag actually used across this book's examples

| Flag | Does | Used in |
|---|---|---|
| `-M <machine>` | Selects the emulated board (`virt`, `pc`, `q35`, `mps2-an385`, ...) | every example |
| `-cpu <model>` | Selects the emulated CPU model | [`hello-kernel`](./systems/07-hello-kernel-overview.md) (`cortex-a72`) |
| `-nographic` | No GUI window — routes the emulated serial console to your terminal | most examples in this book |
| `-serial stdio` | Alternative way to attach the serial console to your terminal | [OxideOS](./oxideos/00-overview.md)'s `run-bios` |
| `-kernel <elf>` | Loads and boots an ELF directly, standing in for a bootloader | [`hello-kernel`](./systems/09-hello-kernel-boot-to-execution.md), embedded blinky |
| `-smp <n>` | How many CPU cores to emulate — relevant to [Multicore & SMP](./asm/10-multicore-and-smp.md) | ARM `virt`/`-smp` config |
| `-m <size>` | How much RAM to give the guest | OxideOS's run targets (`2G`) |
| `-accel hvf` / `-accel kvm` | Hardware-accelerated virtualization instead of software TCG | noted in [`hello-kernel`'s README](./systems/07-hello-kernel-overview.md), OxideOS's `run-kvm-x86_64` |
| `-drive file=...,format=raw` | Attach a disk image | [Systems, Chapter 6](./systems/06-hands-on-building-a-disk-image-in-rust.md)'s `disk.img` |
| `-bios` / `-drive if=pflash,...` | Supply firmware (legacy BIOS blob, or UEFI `OVMF` flash images) | OxideOS's UEFI boot targets |
| `-netdev`/`-device rtl8139` | Attach an emulated network card | OxideOS's networking targets |
| `-s -S` | Start a GDB server, halted at boot | [interactive debugging](#real-use-case-4-interactive-kernel-debugging-via-gdb--arguably-qemus-killer-feature-for-this-exact-kind-of-work), above |
