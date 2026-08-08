# 5. Paging & MMU Setup: the Instructions That Turn On Virtual Memory

[Systems, Chapter 3](../systems/03-ram-and-virtual-memory.md) covered
*what* virtual memory is (pages, frames, page tables). This chapter is
the assembly that actually switches a CPU from "addresses are physical"
to "addresses go through the MMU" — a handful of privileged
instructions, on both architectures, that must run in exactly the right
order or the machine faults immediately.

## x86-64: `CR0`, `CR3`, `CR4`

Three control registers, read/written only via `mov` to/from a
general-purpose register (there's no `mov cr3, <immediate>` form):

```
mov rax, page_table_root_physical_addr
mov cr3, rax          ; CR3: physical address of the top-level page table

mov rax, cr4
or  rax, (1 << 5)      ; CR4.PAE — required for 64-bit paging
mov cr4, rax

mov rcx, 0xC0000080    ; IA32_EFER MSR
rdmsr
or  eax, (1 << 8)       ; EFER.LME — long mode enable
wrmsr

mov rax, cr0
or  rax, (1 << 31)      ; CR0.PG — paging enable
mov cr0, rax
```

- **`CR3`** holds the physical address of the top-level (PML4) page
  table — this one register is *the* pointer that defines a process's
  entire address space; switching it is how
  [Context Switching](./07-context-switching.md) gives each process
  its own memory view.
- **`CR4.PAE`** must be set before enabling long-mode paging — x86-64's
  4-level paging is built on the older PAE (Physical Address Extension)
  mechanism.
- **`EFER.LME`** (a Model-Specific Register, not a control register —
  hence `rdmsr`/`wrmsr` instead of `mov`) enables long mode itself.
- **`CR0.PG`** is the actual on/off switch — setting this bit is the
  instruction after which every subsequent memory access goes through
  the page tables `CR3` points at. Get the order wrong (e.g. set `CR0.PG`
  before `CR3` is valid) and the very next instruction fetch faults.

See [OxideOS Concepts, Chapter 3](../oxideos/oxide_cocepts/03_memory_management.md)
for how `kernel/src/kernel/mem/paging_allocator.rs` builds the page
tables this sequence points at.

## aarch64: `TTBR0_EL1`/`TTBR1_EL1`, `TCR_EL1`, `MAIR_EL1`, `SCTLR_EL1`

More registers, but the same shape — configure everything the MMU needs
to know, *then* flip one enable bit last:

```
msr ttbr0_el1, x0      ; TTBR0_EL1: page table root for user-space (low) addresses
msr ttbr1_el1, x1      ; TTBR1_EL1: page table root for kernel-space (high) addresses
msr tcr_el1, x2        ; TCR_EL1: translation control — page granule size, address widths
msr mair_el1, x3       ; MAIR_EL1: memory attribute encodings (device vs normal memory, caching)
isb                     ; instruction sync barrier — ensure the above are visible before continuing

mrs x4, sctlr_el1
orr x4, x4, #1          ; SCTLR_EL1.M — MMU enable
msr sctlr_el1, x4
isb                     ; and again, since this changes how instruction fetches work
```

- **Two TTBRs, not one** — aarch64 splits the address space in half by
  the top address bit: `TTBR0_EL1` covers low addresses (conventionally
  userspace), `TTBR1_EL1` covers high addresses (conventionally kernel
  space), each with its *own* independent page table root. This is a
  genuine structural difference from x86-64's single `CR3` — switching
  only the user half on a context switch (leaving `TTBR1_EL1` fixed) is
  a natural, hardware-supported optimization aarch64 offers that x86-64
  doesn't have a direct equivalent for.
- **`MAIR_EL1`** has no x86-64 counterpart in this sequence at all — x86
  encodes memory type (cacheable, write-combining, etc.) per page-table
  entry using fixed bit meanings; aarch64 instead has page-table entries
  reference an *index* into `MAIR_EL1`, a small programmable table of
  memory-type definitions, resolved indirectly.
- **`isb`** (Instruction Synchronization Barrier) appears twice —
  aarch64 requires you to explicitly force the pipeline to discard
  anything it may have spoken speculatively before the system-register
  write takes effect. x86-64's control-register writes are architecturally
  guaranteed to take effect for subsequent instructions without an
  explicit barrier — this is the first preview of
  [Chapter 8](./08-atomics-and-memory-barriers.md)'s theme: aarch64
  requires explicit ordering that x86-64 gives you implicitly.

## The common structural pattern

Both sequences: (1) point a register at your page tables, (2) configure
how the MMU should interpret them, (3) flip exactly one enable bit last,
after everything else is in place. Neither architecture lets you enable
paging speculatively "and fix the page tables after" — the page tables
must already correctly map the code currently executing (including the
instruction right after the enable bit flips), or the CPU faults trying
to fetch its own next instruction with nowhere to land.
