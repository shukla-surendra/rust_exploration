# 3. RAM & Virtual Memory

## RAM at the hardware level

Physical RAM is a collection of memory chips (DRAM modules). Each cell
stores a bit, grouped into bytes. The CPU sees RAM as one long array of
bytes, starting at address 0 up to however much is installed — with 4
GB of RAM, that's physical addresses `0x00000000` through `0xFFFFFFFF`.

Unlike disks (chapter 2), there are no sectors or blocks here — RAM is
**byte-addressable** (chapter 1's term), and latency is measured in
nanoseconds rather than the milliseconds a spinning disk needs. Chapter
4 makes this comparison precise; this chapter is purely about how RAM
itself is organized.

## Physical, virtual, and logical addresses — three different things

The CPU doesn't hand raw physical addresses to your program in a modern
OS. Three related but distinct terms:

- **Physical address** — the real hardware location in a RAM chip.
- **Virtual address** — what a running program actually sees and uses.
  Mapped to a physical address by the **MMU** (Memory Management Unit),
  a piece of the CPU built specifically for this translation.
- **Logical address** — occasionally used to mean a segmented address
  (x86 real/protected mode specifically) — a narrower, more historical
  term than the other two.

The practical result: every program believes it has its own private,
continuous chunk of memory (its virtual address space), while the OS +
MMU quietly map that onto whatever physical RAM is actually available
— potentially scattered, potentially even partially swapped to disk,
invisibly to the program. This is precisely the mechanism that makes
[Ownership, Borrowing & Lifetimes](../workbook/02-ownership-borrowing-lifetimes.md)'s
description of the stack and heap possible — "the stack" and "the
heap" are regions *within* a process's virtual address space, not
literal, fixed physical RAM locations; see
[Stack vs Heap](../foundation/stack-vs-heap.md) for how Rust specifically
uses that space.

## Pages and frames — how the mapping is actually organized

The OS doesn't map virtual-to-physical addresses one byte at a time —
that would need an enormous table. Instead it divides memory into
fixed-size chunks:

- **Page** — a chunk of *virtual* address space, commonly 4 KB (can be
  2 MB or 1 GB for "huge pages" in specific workloads).
- **Frame** — a page-sized chunk of *physical* RAM.

```
Virtual Address Space → divided into Pages (4 KB each)
Physical RAM          → divided into Frames (4 KB each)
```

A **page table** (maintained by the OS, consulted by the MMU on every
memory access) records which virtual page maps to which physical
frame — e.g. virtual page `0x0001` → physical frame `0x1000`, virtual
page `0x0002` → physical frame `0x5000`, and so on, no requirement that
consecutive virtual pages map to consecutive physical frames. **Pages
are purely a software/CPU convention** — unlike a disk's sectors
(chapter 2), which are baked into the hardware at manufacture, page
size is a choice the OS and CPU agree on, not a property of the RAM
chips themselves.

## What the bootloader hands the OS: the memory map

At boot, before an OS's own memory management even starts, the
BIOS/UEFI (or a bootloader following the Multiboot specification) hands
the OS a **memory map** — which regions of physical RAM are usable,
which are reserved (BIOS, memory-mapped I/O devices, ACPI tables), and
which are already occupied:

```
0x00000000 - 0x0009FFFF → usable RAM
0x000A0000 - 0x000FFFFF → reserved (video memory, BIOS)
0x00100000 - 0x7FFFFFFF → usable RAM
0x80000000 - ...        → reserved (PCI, devices)
```

This is why a machine with "8 GB installed" often reports only ~7.8 GB
usable — chunks of the physical address space are permanently carved
out for hardware, not available to the OS's general-purpose allocator.

## RAM addresses aren't factory-set — unlike disks

This is the sharpest contrast with chapter 2: a disk *does* come with
fixed logical addresses baked in at manufacture. A RAM stick does
**not** — it's just an array of storage cells with row/column wiring;
it has no idea what address range it will occupy. That's decided at
boot, by the **memory controller** (built into the CPU on modern
systems):

1. On power-up, the memory controller probes each RAM slot via **SPD**
   (Serial Presence Detect — a small chip on each DIMM) to learn its
   capacity, timing, and voltage.
2. It then **maps** each installed module into a portion of the
   system's physical address space, e.g.:
   ```
   0x00000000 – 0x3FFFFFFF → 1 GB module in slot 1
   0x40000000 – 0x7FFFFFFF → 1 GB module in slot 2
   ```
3. If you physically move RAM sticks between slots, this mapping can
   change on next boot.

So while a disk's sector layout is a fixed hardware property you can
inspect the same way on any machine it's plugged into (chapter 2), RAM
address ranges are **per-machine**, decided fresh by that machine's
memory controller and firmware at every boot — which is exactly why
Linux's `/proc/iomem` (or a hand-rolled memory-map parser, next section)
shows memory regions specific to the machine it's running on, not a
property of the RAM stick itself.

## Exploring it: the Rust shape of a memory map

```rust
#[derive(Debug)]
pub struct MemoryRegion {
    pub start: usize,
    pub end: usize,
    pub region_type: MemoryRegionType,
}

#[derive(Debug)]
pub enum MemoryRegionType {
    Usable,
    Reserved,
    Acpi,
    Mmio,
}
```

This is the natural Rust shape for the memory map described above — a
bootloader following the Multiboot spec hands the OS a list of entries
matching this shape, and OS-level code loops through them recording
usable vs. reserved regions. This is the direct RAM equivalent of
reading a disk's partition table (chapter 2's "OS asks the drive how
many logical blocks it has") — same idea, applied to physical RAM
regions instead of disk sectors. On a running Linux userspace system
(rather than inside a bootloader), the closest thing you can inspect
without special privileges is `cat /proc/meminfo`, or `dmesg | grep -i
memory` for the boot-time map the kernel itself received.

## RAM vs. disk, at a glance

| Feature | Disk | RAM |
|---|---|---|
| Unit of addressing | Sectors (512 B / 4 KB) | Bytes |
| Access time | ms (mechanical/SSD) | ns (electrical) |
| CPU access | Via controller/driver (PIO/DMA) | Direct, via the memory bus |
| Address origin | Fixed at manufacture (drive firmware) | Assigned per-boot (memory controller) |
| Mapping | Logical sectors → physical blocks | Virtual addresses → physical RAM frames |

Chapter 4 expands this table into the full picture — adding SSD/flash
internals and the actual *why* behind each row, including the
read/write pattern differences that matter most in practice.
