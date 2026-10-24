# Systems From First Principles — overview

This section is different from the other three in this book. The
[Workbook](../workbook/00-how-to-use.md),
[Foundation](../foundation/crates-and-modules.md), and
[Async Rust](../async-rust/00-is-it-in-the-language-or-not.md) sections
are about **the Rust language**. This section is about **what's
underneath it** — the hardware and OS concepts that exist whether or
not you're using Rust at all (disks, RAM, addressing, the boot process),
built up from first principles, bare metal, with Rust used as the tool
to actually poke at and verify each concept rather than just read about
it.

## Where this content came from

**Storage & Memory** (chapters 1–6) started as working notes in
`use_cases/disk_exploration/docs/` while building the `disk_exploration`
crate — a real, working Rust program in this repo
(`use_cases/disk_exploration/src/`) that opens a file, treats it as a
raw block device, writes an actual boot sector, builds a GPT partition
table, formats a FAT filesystem inside it, and writes a real file to
that filesystem. Chapter 6 walks through exactly how, line by line.

**Kernel & Bare Metal** (chapters 7–10) is imported from a separate
project, `hello-kernel` — a ~50-line freestanding Rust binary that boots
as an actual bare-metal aarch64 "kernel" under QEMU, with no OS, no
bootloader crate, and no standard library. Chapters 7–10 walk through
its entire boot sequence, byte by byte and instruction by instruction.

Both halves earn the "first principles" framing the same way: there's
real, runnable code to point at for nearly every concept covered, not
just prose.

## This is a living section

Unlike the other three sections (each covering a mostly-fixed body of
knowledge), this one is meant to keep growing — the next system-level
topic you dig into from first principles (networking, CPU
caches/pipelining, filesystems in more depth, process scheduling,
whatever comes up next) belongs here as a new chapter. Follow the same
shape the existing chapters use: build up vocabulary before diving in,
verify claims against real Rust code where possible, and end with
something hands-on.

## Reading order

**Storage & Memory:**

1. [Prerequisites: Bits, Bytes & Addressing](./01-prerequisites-bits-bytes-and-addressing.md) — vocabulary everything else assumes
2. [Disks, Sectors & Addressing](./02-disks-sectors-and-addressing.md)
3. [RAM & Virtual Memory](./03-ram-and-virtual-memory.md)
4. [Storage Hierarchy: HDD vs SSD vs Flash vs RAM](./04-storage-hierarchy-hdd-ssd-flash-ram.md) — the comparison chapter tying 2 and 3 together
5. [The Boot Process: BIOS/MBR vs UEFI/GPT](./05-boot-process-bios-uefi.md)
6. [Hands-On: Building a Disk Image in Rust](./06-hands-on-building-a-disk-image-in-rust.md) — the capstone, walking through this repo's actual `disk_exploration` crate

**Kernel & Bare Metal** (the `hello-kernel` case study — can be read
independently of 1–6, though chapters 1, 2, and 5 are referenced along
the way):

7. [Overview: what it is and how to run it](./07-hello-kernel-overview.md)
8. [Build, Editions & the Linker Script](./08-hello-kernel-build-and-linking.md)
9. [From QEMU's `-kernel` Flag to the First Rust Code](./09-hello-kernel-boot-to-execution.md)
10. [UART Output & Panics](./10-hello-kernel-uart-and-panics.md)

**General (can be read any time after chapter 1):**

11. [Talking to Hardware: Port-Mapped I/O vs. Memory-Mapped I/O](./11-talking-to-hardware-pmio-vs-mmio.md) — the two mechanisms behind chapter 10's UART code, and everything else a CPU talks to

If you already know chapter 1's vocabulary cold, skip straight to
chapter 2 — it's there for the same reason
[Chapter 0 of Async Rust](../async-rust/00-is-it-in-the-language-or-not.md)
exists: so later chapters don't have to stop and define terms mid-
explanation.
