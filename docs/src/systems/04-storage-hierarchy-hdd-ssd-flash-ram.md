# 4. Storage Hierarchy: HDD vs SSD vs Flash Drive vs RAM

Chapters 2 and 3 covered disks and RAM separately. This chapter is the
direct comparison — specifically the **read/write pattern** differences,
since that's what actually determines which device is right for which
job, and it's the thing that trips people up most: "it's all just
storage" is not true in any way that matters once you're picking
hardware or reasoning about performance.

## The core distinction: how each technology actually stores a bit

| Technology | How a bit is physically stored | Erasable in place? |
|---|---|---|
| **HDD** | Magnetic orientation of a spot on a spinning platter | Yes — flip the magnetic orientation directly, byte by byte within a sector |
| **SSD / Flash (USB, SD cards)** | Electrical charge trapped in a NAND flash cell | **No** — a cell must be fully erased before it can be rewritten |
| **RAM (DRAM)** | Charge held in a tiny capacitor per cell, needs constant refreshing | Yes — directly overwritten, no erase step, but volatile (chapter 1) |

That "erasable in place: no" row for flash is the single fact
everything else in this chapter follows from.

## Writing style, precisely — this is the part that differs the most

**HDD: overwrite in place.** A magnetic read/write head can flip a
spot's polarity directly, so updating one sector means: move the head
to that sector (seek), wait for the platter to spin the right spot
underneath it (rotational latency), then write. No erase step needed —
you can overwrite a sector as many times as you like, directly.
**Analogy:** a vinyl record groove, or writing in pencil on a physical
notebook page — you can erase and rewrite that exact spot, but there's
real, physical travel time to get the "pen" to the right page and
line first.

**SSD / Flash: erase-before-write, at a bigger granularity than you
write.** NAND flash cells can only transition one direction (say,
1→0) by writing; going back (0→1) requires an **erase**, and erasing
can only be done for a whole **block** at a time — a group of many
**pages**. A page (the read/write unit, ~4 KB) can be written directly
*only if it's already empty*; updating existing data means the
controller must find/allocate a fresh empty page elsewhere, write the
new data there, mark the old page as stale, and erase the old page's
whole block *later*, once enough of it is stale, as a background
operation. **Analogy:** a whiteboard where you can only write, never
erase, one letter — to actually erase anything you have to wipe the
*entire section* it's part of, so a smart eraser (the SSD controller)
quietly copies whatever's still needed elsewhere first, then wipes the
section clean in bulk, later, when convenient.

This mismatch (write in small pages, erase in big blocks) is the root
cause of two terms worth knowing by name:

- **Write amplification** — writing one small page can eventually force
  copying and erasing an entire block's worth of *other*, unrelated
  data around it, so the physical work done is a multiple of the
  logical write you asked for.
- **Wear leveling** — each flash cell can only be erased a limited
  number of times before it wears out, so the controller deliberately
  spreads writes across all cells evenly (rather than hammering the
  same physical spot) to make the whole device wear out roughly evenly
  instead of failing in one worn-out corner first.

**RAM: overwrite in place, no erase step, but it forgets.** RAM has
neither HDD's mechanical seek delay nor SSD's erase-before-write
requirement — any byte can be directly rewritten, immediately, with no
extra bookkeeping. The tradeoff is chapter 1's volatility: DRAM cells
leak their charge and must be electrically *refreshed* thousands of
times per second just to remember what they're storing, and lose
everything the instant power is cut. **Analogy:** a chalkboard grid
where you can walk up and instantly rewrite any single square directly
— but the whole board is wiped clean the moment the lights (power) go
out, which is why nothing here needs wear-leveling: there's no
"remembering for years" property to protect.

## Why this matters for read/write *patterns*, not just raw speed

- **HDDs punish random access, reward sequential access.** Reading one
  big contiguous file is close to the drive's rated throughput; reading
  the same total data scattered across thousands of small random
  locations pays the mechanical seek+rotate cost *per access* — often
  the dominant cost by far. This is chapter 1's IOPS-vs-throughput
  distinction made concrete: an HDD's IOPS number is low specifically
  because of this mechanical penalty.
- **SSDs handle random access far better, but writes still aren't
  "free."** No moving parts means no seek time — random reads are
  nearly as fast as sequential ones. Random *writes*, though, are where
  write amplification (above) bites hardest: many small scattered
  writes generate more background erase/copy work than one large
  sequential write of the same total size.
- **Cheap USB flash drives are the same NAND technology as an SSD, with
  a much weaker controller.** The chip-level physics (pages, blocks,
  erase-before-write) are identical — the practical difference is that
  a budget USB drive's controller often has less sophisticated
  wear-leveling, no TRIM support (the OS telling the drive "this data is
  deleted, you can erase it whenever"), and a much smaller/absent write
  cache — which is why the *same* random-write workload that an SSD
  handles gracefully can make a cheap flash drive feel dramatically
  slower and wear out noticeably faster.
- **RAM has no "access pattern" penalty at all** in the way disks do —
  byte-addressable, direct-overwrite access means random and sequential
  reads/writes cost essentially the same, which is exactly why RAM is
  the right place for data structures with unpredictable access
  patterns (hash maps, pointer-chasing structures) that would be
  painful on any disk-backed storage.

## The full comparison table

| | HDD | SSD | Flash Drive (USB) | RAM |
|---|---|---|---|---|
| Addressable unit (OS view) | Sector (512 B / 4 KB) | Sector (512 B / 4 KB) | Sector (512 B / 4 KB) | Byte |
| Internal write unit | Sector, in place | Page (~4 KB) | Page (~4 KB) | Byte, in place |
| Internal erase unit | N/A (overwrite in place) | Block (~512 KB, must erase before reuse) | Block (~512 KB) | N/A (volatile, not erase-based) |
| Random access penalty | High (seek + rotational latency) | Low | Low–moderate (weaker controller) | None |
| Volatile? | No | No | No | **Yes** |
| Typical latency | milliseconds | tens of microseconds | tens–hundreds of microseconds | nanoseconds |
| Wears out from writes? | No (mechanical wear instead) | Yes (limited erase cycles per cell) | Yes (same, often faster) | No |

## Tying it back to Rust: this is all hidden behind `std::fs`

None of this shows up in ordinary Rust code — `std::fs::write(path,
data)` (or `File::write_all`, used directly in
`use_cases/disk_exploration/src/main.rs`) looks and behaves identically
whether the target is an HDD, SSD, USB stick, or a `tmpfs` RAM-backed
filesystem. That's the whole point of the OS's block-device abstraction
from chapter 2 — `BlockDevice`'s `write_sector` doesn't know or care
which of these technologies is underneath it, and neither does your
Rust code most of the time. This chapter's content becomes practically
relevant specifically when you're **choosing hardware** for a
write-heavy workload, **diagnosing** unexpectedly slow small-write
performance, or — as in chapter 6 — deliberately writing low-level code
that talks in sectors directly instead of going through a filesystem.
