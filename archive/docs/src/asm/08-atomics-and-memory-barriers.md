# 8. Atomics & Memory Barriers: the Sharpest x86-vs-ARM Gotcha

Every earlier chapter's assembly was mostly "different mnemonics, same
idea." This one is a genuine, easy-to-miss correctness trap: **x86-64
and aarch64 have fundamentally different memory ordering models**, and
code that happens to work on one can be silently broken on the other —
a real hazard when porting kernel code between the architectures the
way `hello-kernel`/OxideOS represent.

## Prerequisite: what's actually being kept in sync

Barriers and coherency both operate on the same underlying thing: the
**cache hierarchy** sitting between a core and RAM. Worth being
concrete about before the rest of this chapter talks about "ordering"
and "visibility" in the abstract.

Every load/store a core executes is checked against this hierarchy
automatically, in hardware, on every single access — never something
software explicitly requests:

```
core → L1 (private, ~4 cycles) → L2 (private, ~12 cycles) → L3/LLC (shared, ~30-40 cycles) → RAM (hundreds of cycles)
```

- **L1** — smallest and fastest, split into L1i (instructions) and L1d
  (data), private to each core.
- **L2** — bigger, slower, still private to each core in most designs.
- **L3 (last-level cache, LLC)** — much bigger, slower still, and —
  per [Multicore & SMP](./10-multicore-and-smp.md#whats-private-per-core-and-whats-actually-shared) —
  **shared across every core on the chip**. This is the shared cache
  that chapter's private/shared inventory referred to.

Caches don't store individual bytes — they store fixed-size **cache
lines**, 64 bytes on both x86-64 and aarch64. Reading one byte pulls in
the entire 64-byte line containing it. Each cached line carries a tag
(which address range it represents), a valid bit, and — the part that
matters for this chapter — a **coherency state**, tracked by a protocol
called **MESI** (Modified / Exclusive / Shared / Invalid; vendor
variants like MESIF/MOESI add a state or two but work the same way):
when one core writes to a line, every other core's cached copy of that
*same line* is automatically invalidated, forcing them to re-fetch the
current value on next access.

**This is exactly the split this chapter is built on:** MESI coherency
is what guarantees a write eventually becomes visible to other cores at
all — but it says nothing about *when*, relative to a core's *other*
memory operations. That "when, relative to what else" question is
precisely what barriers (below) control. Coherency without barriers
still leaves you with a correct-eventually-but-unordered view of memory
— which is exactly the "weakly ordered" behavior aarch64 exposes by
default and x86-64 mostly hides.

**One concrete, easy-to-hit consequence worth knowing by name: false
sharing.** Two logically unrelated variables that happen to land in the
same 64-byte cache line get bounced between cores' caches on every
write to *either* one — full MESI invalidation traffic for data that
was never actually shared between the threads touching it. This is a
real, non-obvious performance bug in concurrent Rust code (two
`AtomicU32`s in the same struct, hammered by different threads, can be
dramatically slower than the same two fields padded onto separate cache
lines) — worth knowing the mechanism even though this chapter's `asm!`
examples don't need to address it directly.

## The core difference

- **x86-64 is strongly ordered** (specifically, mostly TSO — Total
  Store Order). Ordinary loads and stores from different CPUs largely
  appear in a globally consistent order without any extra instructions;
  you mostly only need special handling for read-modify-write atomics
  (below).
- **aarch64 is weakly ordered.** The CPU (and compiler) are free to
  reorder, cache, and delay memory operations far more aggressively
  unless you *explicitly* insert a barrier — the same "the compiler
  doesn't know this needs to happen in order" problem
  [Systems, Chapter 10](../systems/10-hello-kernel-uart-and-panics.md)
  covered for `read_volatile`/`write_volatile`, except barriers address
  *hardware* reordering (across CPU cores, or between the CPU and a
  device), which `volatile` alone does not solve.

This is precisely why `volatile` accesses (correct for `hello-kernel`'s
single-core UART polling) aren't automatically sufficient once multiple
CPU cores are involved, or a device's own internal state can change
memory outside the CPU's own instruction stream — `volatile` only
promises the compiler won't reorder/elide *this* core's own accesses;
it says nothing about what a *different* core or device observes, and
in which order.

## Rust's atomics already handle most of this — if you use them

```rust
use core::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);

COUNTER.fetch_add(1, Ordering::SeqCst);
let value = COUNTER.load(Ordering::Acquire);
```

`core::sync::atomic` types compile to the *correct* instructions for
whichever architecture you're targeting — `Ordering::SeqCst` on x86-64
might need almost no extra work beyond a `lock`-prefixed instruction;
the identical Rust code on aarch64 compiles in the barrier instructions
below automatically. **This is the main practical lesson of this
chapter: prefer `core::sync::atomic` over hand-rolled asm for ordinary
atomic operations** — it's exactly the kind of cross-architecture
correctness problem you don't want to solve by hand per-target.

The chapters in this section reach for raw asm specifically for things
`core::sync::atomic` doesn't cover — a context switch's register
save/restore, an MMU enable sequence — where a barrier still needs to
be inserted explicitly even though it's not "an atomic operation" in
Rust's type-level sense.

## The instructions themselves, for when you do need them directly

**x86-64: the `lock` prefix, and rarely anything else**

```
lock xadd [counter], eax     ; atomic fetch-and-add
lock cmpxchg [counter], ebx  ; atomic compare-and-swap
mfence                        ; full memory fence — rarely needed given TSO
```

`lock` makes a single read-modify-write instruction atomic across
cores. Explicit fences (`mfence`/`sfence`/`lfence`) are comparatively
rare in ordinary x86 kernel code specifically *because* the baseline
ordering is already strong.

**aarch64: barriers are load-bearing, not optional extras**

```
dmb sy      ; Data Memory Barrier — orders memory accesses before/after it
dsb sy      ; Data Synchronization Barrier — like dmb, but also waits for completion
isb         ; Instruction Synchronization Barrier — flushes the pipeline (Chapter 5)
```

- **`dmb`** — "every memory access before this point is visible to
  other observers before any memory access after this point" — but
  doesn't wait for those accesses to actually *finish*, just orders
  them.
- **`dsb`** — the stronger version: also blocks the current core until
  prior memory accesses have genuinely completed. Required, for
  instance, right after the MMU-enable sequence in
  [Chapter 5](./05-paging-and-mmu-setup.md) alongside `isb`, to
  guarantee the page-table writes that configured the MMU are actually
  visible before relying on translations through it.
- aarch64's load/store instructions also have acquire/release *variants*
  built directly into the instruction (`ldar`/`stlr` — "load-acquire",
  "store-release") rather than requiring a separate barrier instruction
  for the common producer/consumer pattern — this is what
  `Ordering::Acquire`/`Ordering::Release` in Rust's atomics compile down
  to on this architecture, versus x86-64 where acquire/release
  semantics are close to free given the baseline strong ordering.

## Why this matters even for single-core code

Barriers aren't only about multiple CPU cores — they also govern
ordering between the CPU and **devices** (a UART, a disk controller)
and, per [Chapter 5](./05-paging-and-mmu-setup.md), between writing
configuration to a system register and that configuration actually
taking effect for subsequent instructions. A single-core, single-task
kernel like `hello-kernel` still needs `isb` after enabling the MMU for
exactly this reason — "single core" doesn't mean "no reordering
possible," it just removes the *cross-core* half of the problem, not
the CPU-vs-memory-subsystem half.
