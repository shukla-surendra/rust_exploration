# OxideOS — Deep Dive

> Imported from a separate project, **OxideOS** — a full, from-scratch
> `no_std` Rust operating system (x86-64, Limine bootloader, BIOS + UEFI,
> Ring 3 userspace, GUI desktop, TCP/IP, musl libc, Bash, Python 3, Lua,
> BusyBox — see its own `README.md` for the full picture). Only the
> project's own **tutorial documentation** is brought in here — its two
> "how it works, step by step" doc sets, exactly as OxideOS's own
> top-level `README.md` distinguishes them from its separate
> *design-rationale* docs (`docs/boot.md`, `docs/interrupts.md`,
> `docs/memory.md`, `docs/scheduler.md`, `docs/filesystem.md` — short
> "why, not how" notes, not imported) and its ARM-port-status docs
> (`docs/arm/` — not imported; a handful of cross-references to it below
> are left as plain text rather than dangling links).

If [Systems From First Principles](../systems/00-overview.md) was
"first principles, one focused concept at a time, with small runnable
examples" — this section is the opposite end of the same spectrum:
**a complete, working, full-featured OS**, with two different
documentation sets guiding you through understanding it.

## The two doc sets, and how they differ

- **[Concepts Walkthrough](./oxide_cocepts/README.md)** — six chapters,
  read in order, each explaining one subsystem's *architecture and
  design*: boot, interrupts/CPU setup, memory management, syscalls/user
  mode, graphics/GUI, storage. Prose-first, code cited as evidence.
- **[Study Journal](./study/README.md)** — eight chapters (00–07) plus a
  19-document hardware reference library. Explicitly *hands-on*: each
  doc "points you at the exact code to read, and ends with exercises to
  do yourself" (its own README's words). This is the one to work
  through if you want to come out the other end able to extend the OS
  yourself, not just understand it conceptually.

They cover overlapping ground from different angles — e.g. memory
management has both a Concepts chapter (design/architecture) and a
Study Journal chapter (bump allocator + page tables, exercise-driven),
and each cross-references the other.

## Where to start

- **New to reading OS-level Rust code at all?** Start with
  [Study Journal 00 — Rust for OS Readers](./study/00_rust_for_os_readers.md) —
  it exists specifically for that gap.
- **Want the architecture picture first, code second?** Start with
  [Concepts Walkthrough](./oxide_cocepts/README.md), chapter 1.
- **Want to trace one concrete thing end to end** (e.g. "what actually
  happens when I press a key")? Jump straight to
  [Study Journal 03 — Keypress Trace](./study/03_keypress_trace.md).
- **Looking for a specific piece of hardware** (a PIC, a UART, an NVMe
  drive, a modern GPU)? Go straight to the
  [Hardware Reference](./study/hardware/README.md) — it's a lookup
  library, not something meant to be read start to finish.

Both doc sets constantly cite exact file paths inside OxideOS's own
`kernel/src/` tree (e.g. `kernel/src/kernel/mem/paging_allocator.rs`) —
those are plain source-code references into the OxideOS project, not
part of this repo, and are left as-is (not linked) throughout.
