# 10. Multicore & SMP: Who Wakes the Other Cores, and How

Every earlier chapter in this section describes **one CPU core**, from
the instant it starts executing to running a full scheduler. A modern
machine has several (often many) cores, and every chapter so far has
been silently describing what happens on exactly one of them — this
chapter is the missing piece: how the others get started, who's
responsible, and how much of it is unavoidably assembly.

## The short answer, up front

- **Firmware discovers how many cores exist and tells the OS** — the
  OS doesn't probe for cores itself.
- **The OS decides when/whether to start them**, using a firmware- or
  architecture-defined mechanism — a small, specific, and yes,
  assembly-dependent handshake.
- **x86-64 needs real assembly for this that no other chapter needs** —
  a secondary core wakes up in 16-bit real mode, even on a 64-bit
  system, which Rust cannot generate code for at all. aarch64 needs
  almost none — one privileged instruction.
- **Neither `hello-kernel` nor OxideOS implements this yet** — OxideOS's
  own roadmap lists SMP as a future milestone, not current behavior;
  everything in this book up to this point, including every other
  assembly chapter, is single-core. This chapter describes the
  mechanism, not something already running in this repo's code.

## Step 1: discovery — firmware tells the OS what exists

**x86-64**: the **ACPI MADT** (Multiple APIC Description Table), part of
the ACPI tables firmware hands the OS at boot, lists every CPU core's
**Local APIC ID** — a unique per-core identifier the OS reads to learn
how many cores exist and how to address each one individually for the
next step. The OS parses this table; it never has to guess or probe
hardware to find cores.

**aarch64**: either an ACPI MADT-equivalent (GICC entries) or a
**Device Tree** (`cpu` nodes under `/cpus`), depending on the platform
— QEMU's `virt` machine (the same machine `hello-kernel` boots on, per
[Systems, Chapter 9](../systems/09-hello-kernel-boot-to-execution.md))
supports both, configurable via QEMU's `-smp` flag and exposed to the
guest either way.

## Step 2: waking a secondary core — the part that actually needs assembly

The terminology: the core that's running your kernel already (the one
that executed `_start`) is the **BSP** (Bootstrap Processor); every
other core is an **AP** (Application Processor), sitting halted,
waiting to be told where to start executing.

### x86-64: the INIT-SIPI-SIPI sequence

The BSP sends a specific sequence of **Inter-Processor Interrupts**
(IPIs — covered generally in [Step 4](#step-4-inter-processor-interrupts-ipis) below) to a target AP's Local APIC:

```
1. Send INIT IPI       — resets the target AP to a known, halted state
2. (wait ~10ms)
3. Send SIPI (Startup IPI), with a target vector V
4. (wait, retry SIPI if the AP hasn't started)
```

The **SIPI vector** `V` is not an address — it's a page number
(`V × 0x1000`), and it must point at code located **below the 1MB
mark**, because of the one genuinely unavoidable piece of legacy this
whole process carries: **the AP starts executing in 16-bit real mode**,
the exact same mode x86 CPUs have booted into since the original IBM
PC, regardless of the fact that the BSP has been running 64-bit long
mode (per [Assembly, Chapter 5](./05-paging-and-mmu-setup.md)'s
`EFER.LME` bit) for the entire time since the OS booted. This is why it
needs its own **trampoline** — a small blob of hand-written 16-bit
assembly, placed at a fixed low-memory address before any AP is woken,
that does — on that specific core, independently — the exact same
sequence [Assembly, Chapters 3 and 5](./03-privilege-levels-and-mode-transitions.md)
described for the BSP's own original boot: enable protected mode, set
up a minimal GDT, enable long mode, enable paging (pointing `CR3` at
the *same* page tables the BSP already built), and only then jump into
64-bit Rust code.

**Why this specifically can't be written in `asm!` or `global_asm!`
directly:** Rust (via LLVM) only ever generates code for the mode the
whole kernel is compiled for — there's no way to ask `rustc` to emit
16-bit real-mode instructions for one small blob. The trampoline is
written as raw machine code (hand-assembled, or via a separate `nasm`
invocation producing a flat binary that gets linked in or copied to its
fixed address at runtime) — one of the very few places in a modern OS
where Rust's toolchain genuinely cannot reach at all, not even through
`unsafe`.

### aarch64: PSCI `CPU_ON` — one instruction

aarch64 has no real-mode legacy — a secondary core can be told to start
directly at a full 64-bit address, in the correct exception level,
using the same **PSCI** (Power State Coordination Interface) firmware
already referenced for shutdown in
[Systems, Chapter 1's ARM footnotes](../systems/07-hello-kernel-overview.md):

```rust
unsafe {
    asm!(
        "mov x0, #0xc4000003",   // PSCI CPU_ON function ID
        "mov x1, {target_cpu}",  // which core to wake (its MPIDR_EL1 affinity value)
        "mov x2, {entry_addr}",  // where it should start executing
        "mov x3, {context_id}",  // an arbitrary value passed through to the target
        "smc #0",                 // Secure Monitor Call — hands off to firmware
        target_cpu = in(reg) cpu_id,
        entry_addr = in(reg) entry_point,
        context_id = in(reg) 0,
    );
}
```

One `smc` (or `hvc`, depending on how firmware is configured) instruction
— firmware handles the actual low-level core wake-up, and the target
core starts running your Rust `entry_point` directly, no trampoline, no
mode transition, no separate assembly blob needed. This is a genuinely
large asymmetry between the two architectures: x86-64's SMP bring-up is
one of the hairiest corners of kernel assembly that exists; aarch64's is
a single privileged call.

## What's private per core, and what's actually shared

Before Step 3 lists what each core needs its *own* copy of, it's worth
being explicit about the full picture — this is the split every item in
that list, and every synchronization primitive in Step 4 and
[Chapter 8](./08-atomics-and-memory-barriers.md), ultimately exists to
manage.

**Private to each core** — genuinely separate hardware, one full copy
per core, nothing shared:

- The entire register file: general-purpose registers, program counter,
  stack pointer, and every control/system register this whole section
  has covered (`CR0`–`CR4` per x86-64 core; `TTBR0_EL1`, `SCTLR_EL1`,
  etc. per aarch64 core). Core 1 reading `rsp` gets *its own* stack
  pointer, with no knowledge of what core 0's `rsp` currently holds.
- The Local APIC (x86) / GIC CPU-interface (aarch64) — the part of the
  interrupt controller that actually *receives and acknowledges*
  interrupts for that specific core (as opposed to the shared
  routing/distributor logic, below).
- The **TLB** (Translation Lookaside Buffer) — each core caches its own
  recent virtual→physical translations independently. This one is
  subtle enough to deserve its own explanation in
  [Step 4](#tlb-shootdowns-why-a-shared-page-table-isnt-enough) below,
  since it's *derived from* shared data but doesn't automatically stay
  in sync with it.

**Shared across all cores** — one copy, all cores see the same thing:

- **Physical RAM** — the obvious one, and the reason Step 4 and
  [Chapter 8](./08-atomics-and-memory-barriers.md)'s locks/atomics/
  barriers exist at all: this is what cores actually contend over.
- **The last-level cache (L3) and the interconnect/memory controller**
  connecting cores to RAM — L1, and often L2, are per-core; L3 is
  typically shared, and is what makes cache *coherency* (one core's
  write becoming visible to another) work without every access going
  all the way to RAM. Full hierarchy, cache-line size, and the MESI
  coherency protocol behind this bullet:
  [Atomics & Memory Barriers, "Prerequisite: what's actually being kept
  in sync"](./08-atomics-and-memory-barriers.md#prerequisite-whats-actually-being-kept-in-sync).
- **I/O devices** — one disk controller, one NIC. Only one core's
  kernel code should be touching a given device's registers at a time,
  which is why device drivers need locking even though there's only
  ever "one" of the device in question — the same category of hazard
  as two cores racing on a RAM location, just applied to MMIO instead.
- **The kernel's own page-table data in RAM** — as opposed to the
  *register* pointing at it (`CR3`/`TTBR0_EL1`, private, per Step 3
  below): if every core maps kernel space identically (the normal
  design), that mapping *data* is one shared structure in RAM that
  every core's page-table walk reads.
- **The interrupt controller's shared routing/distributor logic** (the
  I/O APIC on x86, the GIC *distributor* on aarch64) — decides which
  core a given device interrupt gets delivered to; distinct from each
  core's own private Local APIC/CPU-interface, above, which is what
  actually receives it.
- Firmware/ACPI tables, the MADT, device tree — read-only, identical
  for every core reading them.

The short version: **registers are private; RAM, last-level cache,
devices, and page-table contents are shared** — and the entire reason
Step 3 and Step 4 exist is that "shared" doesn't mean "automatically
kept in sync everywhere" — a core's *cached view* of shared data (its
TLB, and to a lesser extent its private caches) can silently go stale,
which is exactly what TLB shootdowns, below, exist to fix.

## Step 3: what a newly-woken core needs before it's actually usable

Landing in 64-bit Rust code isn't the end — each AP independently needs
its own copy of several things every earlier chapter assumed existed
exactly once:

- **Its own stack** — every core executing the same entry code with the
  *same* stack pointer would corrupt each other's data on the first
  function call; each AP must be handed a distinct stack region (per
  [Assembly, Chapter 1](./01-inline-asm-in-rust.md)'s `_start`
  discussion, just N times instead of once).
- **Its own per-CPU data area** — "which core am I, and what task am I
  currently running" can't be a single global variable once multiple
  cores are asking simultaneously. x86-64 conventionally uses the `GS`
  segment base (via `wrmsr` to `IA32_GS_BASE`) pointing at a
  per-core struct; aarch64 uses `TPIDR_EL1`, a system register whose
  entire purpose is holding exactly this "current core's private data
  pointer."
- **x86-64 specifically: its own GDT and TSS** — per
  [Assembly, Chapter 3](./03-privilege-levels-and-mode-transitions.md),
  the TSS holds the kernel-stack-to-switch-to on a privilege-level
  change; that must be a *per-core* value (core 0 handling an interrupt
  must not switch to core 1's kernel stack), so each core loads its own
  TSS via its own `ltr`, even though every core can share the same
  *code* for the GDT-setup routine.
- **Its own copy of `CR3`/`TTBR0_EL1`** (page-table root) — usually the
  *same* page tables as the BSP (a shared kernel address space), loaded
  independently per [Assembly, Chapter 5](./05-paging-and-mmu-setup.md)'s
  sequence, on each core, since `CR3`/`TTBR0_EL1` are themselves
  per-core registers, not shared state.

## Step 4: Inter-Processor Interrupts (IPIs)

Once multiple cores are running, they need a way to interrupt *each
other* — the concrete motivating example is the TLB shootdown described
next.

| | x86-64 | aarch64 |
|---|---|---|
| Mechanism | Write to the Local APIC's **ICR** (Interrupt Command Register) | Write to the GIC's **SGI** (Software Generated Interrupt) register |
| Targeting | By Local APIC ID, or "all others," or "all including self" | By core affinity value, similarly flexible |
| Handling | An ordinary entry in the IDT ([Assembly, Chapter 4](./04-interrupts-and-exceptions.md)) on the receiving core | An ordinary entry in the exception vector table ([Assembly, Chapter 4](./04-interrupts-and-exceptions.md)) on the receiving core |

Mechanically, an IPI is "just" another interrupt from the receiving
core's point of view — it reuses every mechanism
[Chapter 4](./04-interrupts-and-exceptions.md) already built; the only
new part is the *sending* side (writing to the ICR/GIC instead of
waiting for hardware to raise the interrupt automatically).

### TLB shootdowns: why a shared page table isn't enough

The concrete, universal reason every SMP kernel needs IPIs at all —
worth walking through fully, since it's the clearest example of the
"shared data, but cached-per-core knowledge of it" hazard flagged
above.

The **TLB** caches recent virtual→physical address translations
directly in each core, so most memory accesses don't have to walk the
full page-table structure in RAM every time — a real, significant
performance win, and, per the private/shared split above, a genuinely
*per-core* cache, not a shared one.

Now suppose core 0 unmaps a page — say, freeing memory that was mapped
at some virtual address `V` — by updating the (shared, in-RAM) page
table. That page-table *data* is shared, so in principle every core
could see the update immediately. But if core 1 had recently accessed
`V` and cached its old translation in its own TLB, **core 1 has no way
to know the page table changed** — cache coherency (the shared-L3/
interconnect machinery from the private/shared list above) keeps *RAM
contents* consistent across cores, but a TLB is not a cache of RAM
contents in that sense; it's a core-local cache of a *computed result*
(a translation), and nothing about coherency hardware automatically
invalidates it when the underlying page table changes.

Left alone, core 1 would keep using the stale mapping — reading or
writing through an address the OS believes is already freed and
possibly reallocated to something else entirely. The fix is exactly
the IPI mechanism above, used for a specific purpose:

1. Core 0 updates the shared page table (removes the mapping for `V`).
2. Core 0 flushes **its own** TLB entry for `V` —
   `invlpg [V]` (x86-64) or `tlbi vae1, {V}` (aarch64) — cheap,
   local, no IPI needed for this part.
3. Core 0 sends an IPI to every *other* core that might have cached
   that translation (often "all other cores," since tracking exactly
   which cores touched which address is usually not worth the
   bookkeeping).
4. Each receiving core's IPI handler runs the same local flush
   instruction (`invlpg`/`tlbi`) for `V` on itself.
5. Core 0 waits (spins, using the same atomics/barriers primitives from
   [Chapter 8](./08-atomics-and-memory-barriers.md)) until every target
   core has acknowledged completing its flush, *before* considering the
   unmap operation finished — otherwise core 0 might, say, hand that
   physical page to a new allocation while another core could still
   read/write it through the stale mapping.

This is precisely why it's called a "shootdown" — one core is actively
reaching out and forcing every other core to discard specific cached
state, rather than passively waiting for coherency hardware to handle
it, because for a TLB, no such hardware mechanism exists.

## Step 5: what actually changes in the OS once cores can run in parallel

This is the point where
[Atomics & Memory Barriers](./08-atomics-and-memory-barriers.md) stops
being "good practice for correctness" and starts being "the only thing
standing between you and real, observable data corruption":

- **Every shared kernel data structure needs real synchronization** —
  spinlocks (built directly on Chapter 8's `lock`-prefixed/`ldxr`-`stxr`
  atomic instructions), not just `cli`/`daifset` (disabling interrupts
  only protects against *this core* being interrupted — it does nothing
  to stop a *different* core from touching the same data at the same
  moment).
- **The scheduler must become SMP-aware** — either one global run queue
  protected by a lock (simple, but that lock becomes a bottleneck as
  core count grows) or per-core run queues with occasional load-
  balancing/migration between them (what most production kernels
  eventually do).
- **The physical memory allocator must be thread-safe** — two cores
  calling into it simultaneously must not hand out the same physical
  frame twice; see
  [OxideOS Concepts, Chapter 3](../oxideos/oxide_cocepts/03_memory_management.md)
  for the single-core version this would need to be extended from.
- **Rust's `Send`/`Sync`** ([Concurrency, Workbook Chapter 8](../workbook/08-concurrency.md#send-and-sync--the-traits-that-make-all-of-this-enforceable))
  go from "correct in principle, largely moot in practice" to load-
  bearing the instant a second core is actually executing kernel code
  concurrently — a single-core kernel with interrupts is technically
  "concurrent" (an interrupt can interleave with normal code) but never
  truly *parallel* (only one instruction stream ever executes at any
  instant); real SMP is the first point where two threads of kernel
  code can genuinely run at the exact same moment, which is precisely
  the hazard `Send`/`Sync` and the atomics/barriers from
  [Chapter 8](./08-atomics-and-memory-barriers.md) exist to make safe.

## Direct answer: do you need assembly for this?

- **x86-64: yes, unavoidably**, for the real-mode AP trampoline
  specifically — there is no way around it short of firmware doing
  something unusual on your behalf, because 16-bit real mode is simply
  outside what Rust's compiler backend can target. Everything *after*
  the trampoline reaches 64-bit code (per-core stack/GDT/TSS/paging
  setup) reuses ordinary `asm!`/Rust, the same techniques earlier
  chapters already cover.
- **aarch64: barely** — one `smc`/`hvc` instruction via `asm!`
  ([Chapter 1](./01-inline-asm-in-rust.md)) to invoke PSCI `CPU_ON`;
  everything else can be ordinary Rust from the moment the new core
  starts.
- **Coordination once cores are running** (spinlocks, IPIs) needs no
  *new* assembly primitives beyond what
  [Chapter 8](./08-atomics-and-memory-barriers.md) already covers — SMP
  doesn't introduce a new category of instruction, it just makes the
  ones already discussed there mandatory rather than theoretically
  correct.
