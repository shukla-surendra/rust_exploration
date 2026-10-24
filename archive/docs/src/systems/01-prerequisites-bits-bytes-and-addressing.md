# 1. Prerequisites: Bits, Bytes & Addressing

Vocabulary the rest of this section leans on constantly. If any of these
terms are already solid, skip ahead — this chapter is here so later
chapters don't have to stop and define them mid-explanation.

## Bit, byte, and why 8

A **bit** is one binary digit — 0 or 1, the smallest unit of
information a computer stores. A **byte** is 8 bits grouped together —
not a law of physics, just the size the industry converged on decades
ago because it's enough to represent a single character (ASCII) and
divides cleanly into the word sizes CPUs use. Rust makes this concrete:
`u8` is Rust's byte type — literally an unsigned 8-bit integer, values
0–255, and it's what every "raw bytes" API in Rust actually hands you
(`&[u8]`, `Vec<u8>`, `std::fs::read` returning `Vec<u8>`).

```rust
println!("{}", std::mem::size_of::<u8>());   // 1  — one byte, by definition
println!("{}", std::mem::size_of::<u32>());  // 4  — 32 bits = 4 bytes
println!("{}", std::mem::size_of::<u64>());  // 8  — 64 bits = 8 bytes
```

`std::mem::size_of::<T>()` returns a type's size **in bytes** — worth
running for a few types to make "a `u64` is 8 bytes" a felt fact rather
than a memorized one, since the rest of this section talks in bytes,
kilobytes, and sector/page sizes constantly.

## Addressing — what a "memory address" actually is

An **address** is just a number that identifies *which* byte (or, for
disks, which fixed-size chunk — chapter 2) you mean, out of the whole
available space. `0x1000` and `4096` are the same address, just written
in hex vs decimal — hex is conventional here because memory sizes are
powers of two, and hex digits map cleanly onto groups of 4 bits.

**Addressable unit** is the smallest thing a single address can point
to — this differs by hardware, and it's the single fact this whole
section keeps coming back to:

| Hardware | Smallest addressable unit |
|---|---|
| RAM | 1 byte (chapter 3) |
| Disk (HDD/SSD, as seen by the OS) | 1 sector — 512 bytes or 4 KB (chapter 2) |
| Flash memory (SSD internals) | 1 page — often 4 KB (chapter 4) |

A `u64` address can (in principle) point at 2^64 distinct addressable
units — whether that's 2^64 individual bytes (RAM) or 2^64 individual
512-byte sectors (a disk) changes how much total space that address
range actually covers, which is exactly why disks and RAM "count"
differently even when using similarly-sized address values.

## Latency vs. throughput vs. IOPS — three different "speed" numbers

These get conflated constantly, and chapter 4's HDD/SSD/RAM comparison
only makes sense once they're separated:

- **Latency** — how long *one* operation takes to even start returning
  data, measured in time (nanoseconds for RAM, microseconds/milliseconds
  for SSD/HDD). The thing that hurts most when it's high: waiting.
- **Throughput** — how much data moves per second once transfer is
  underway (MB/s, GB/s). A device can have high throughput but still
  feel slow for small operations if its *latency* is high — throughput
  is about sustained large transfers, latency is about the "first byte"
  delay.
- **IOPS** (I/O Operations Per Second) — how many *distinct* read/write
  operations a device can service per second, which matters most for
  lots of small, scattered operations (a database doing many small
  random reads) rather than one big sequential file copy. Low latency
  and high IOPS tend to go together; high throughput doesn't guarantee
  either.

A concrete intuition, expanded fully in chapter 4: an HDD can have
decent throughput (reading one big sequential file) but terrible IOPS
(many small random reads each pay the mechanical seek-time cost) — SSDs
fix the IOPS problem specifically because there's no physical arm that
needs to move.

## Volatile vs. non-volatile

**Volatile** memory loses its contents the instant power is cut — RAM
is volatile, which is *why* it needs no wear-leveling or erase cycles
(chapter 4) and can be byte-addressable and blazing fast: there's no
long-term physical durability to engineer for. **Non-volatile** storage
keeps its contents without power — disks and flash memory (SSDs, USB
drives) are non-volatile, and that durability requirement is the root
cause of nearly every complication chapter 4 covers (sectors, pages,
blocks, wear-leveling) that RAM simply doesn't need to deal with.

## Why this vocabulary earns its own chapter

Every term above reappears, unexplained, starting in the very next
chapter — "sector," "LBA," "page," "byte-addressable," "IOPS," and
"volatile" are used as settled vocabulary from here on, the same way
[Ownership, Borrowing & Lifetimes](../workbook/02-ownership-borrowing-lifetimes.md)
front-loads Rust's ownership vocabulary before every later chapter
leans on it.
