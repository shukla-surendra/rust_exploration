# 9. Full Reference Checklist: Everything Assembly an OS Needs

Every piece of raw assembly a kernel genuinely needs, one table, both
architectures, linked back to the chapter that explains it and the real
code (`hello-kernel` or `OxideOS`) that demonstrates it.

| # | What it does | x86-64 | aarch64 | Chapter | Real code |
|---|---|---|---|---|---|
| 1 | Establish a stack before anything else can run | `mov rsp, stack_top` | `adrp`/`add`/`mov sp, x0` | [1](./01-inline-asm-in-rust.md), [2](./02-registers-and-calling-conventions.md) | `hello-kernel`'s `_start` |
| 2 | Enter a higher privilege level | (n/a — starts at Ring 0 / boot firmware hands off already privileged) | (n/a — QEMU `-kernel` starts at EL1 directly) | [3](./03-privilege-levels-and-mode-transitions.md) | — |
| 3 | Set up privilege-level descriptors | GDT + TSS (`lgdt`, `ltr`) | *(none needed — no segmentation)* | [3](./03-privilege-levels-and-mode-transitions.md) | OxideOS Concepts ch. 2 |
| 4 | Drop to userspace | build a fake interrupt frame + `iret` | set `ELR_EL1`/`SPSR_EL1` + `eret` | [3](./03-privilege-levels-and-mode-transitions.md) | OxideOS Study 05 |
| 5 | Register interrupt/exception handlers | `lidt` (load IDT) | write handler addresses into the fixed 16-slot vector table | [4](./04-interrupts-and-exceptions.md) | OxideOS Study 02 |
| 6 | Save/restore state on interrupt entry | `push` every caller-saved reg, `iretq` to return | `stp`/`ldp` pairs, `eret` to return | [4](./04-interrupts-and-exceptions.md) | OxideOS Study 02 |
| 7 | Enable/disable interrupts | `sti` / `cli` | `msr daifclr, #0xf` / `msr daifset, #0xf` | [4](./04-interrupts-and-exceptions.md) | both |
| 8 | Enable paging / the MMU | set `CR3`, `CR4.PAE`, `EFER.LME`, `CR0.PG` (in that order) | set `TTBR0/1_EL1`, `TCR_EL1`, `MAIR_EL1`, then `SCTLR_EL1.M` + `isb` | [5](./05-paging-and-mmu-setup.md) | OxideOS Concepts ch. 3 |
| 9 | Enter the kernel from a syscall | `int 0x80` or `syscall` | `svc #0` | [6](./06-syscall-entry-exit.md) | OxideOS Concepts ch. 4 |
| 10 | Save/restore state on syscall entry | save `rcx` (return addr) + caller-saved regs | save `x30`/`ELR_EL1` + registers | [6](./06-syscall-entry-exit.md) | OxideOS Study 06 |
| 11 | Switch from one task to another | callee-saved regs + `rsp` + `cr3`, then `ret` | callee-saved regs + `sp` + `x30` + `TTBR0_EL1`, then `ret` | [7](./07-context-switching.md) | OxideOS Study 05 |
| 12 | Atomic read-modify-write | `lock`-prefixed instruction | `ldar`/`stlr` or `ldxr`/`stxr` load/store-exclusive pairs | [8](./08-atomics-and-memory-barriers.md) | `core::sync::atomic`, either arch |
| 13 | Order memory across cores/devices | mostly implicit (TSO); `mfence` when needed | `dmb`/`dsb`/`isb`, required explicitly | [8](./08-atomics-and-memory-barriers.md) | — |
| 14 | Read raw hardware I/O (port-mapped) | `in`/`out` | *(no port I/O — MMIO only, ordinary loads/stores)* | [Systems ch. 10](../systems/10-hello-kernel-uart-and-panics.md) | `hello-kernel`'s UART |
| 15 | Halt / wait for an event (power-saving idle) | `hlt` | `wfe` / `wfi` | — | `hello-kernel`'s idle loop |

## What's deliberately *not* in this reference

- **Booting before `_start` runs at all** — that's firmware/bootloader
  territory (BIOS/UEFI, Limine, U-Boot), not kernel assembly. See
  [Systems, Chapter 9](../systems/09-hello-kernel-boot-to-execution.md)'s
  "why isn't a bootloader needed" for exactly where that boundary sits.
- **SMP (multi-core) bring-up** — waking secondary cores (`INIT`/`SIPI`
  on x86, `PSCI` calls on aarch64) is a real and substantial topic on
  its own, not covered by any chapter above; OxideOS's own roadmap
  lists SMP as a future milestone, not yet implemented.
- **Floating-point/SIMD context** (`fxsave`/`xsave` on x86-64, the `V`
  register file on aarch64) — only needed if a kernel lets userspace use
  floating point across a context switch; both `hello-kernel` and
  OxideOS's current scheduler save only the integer register set from
  [Chapter 2](./02-registers-and-calling-conventions.md)'s table.

## How to actually use this table

Read top to bottom once, linearly — it's the same order a kernel boots
in: establish a stack (row 1) → set up privilege/interrupt
infrastructure (rows 3–7) → enable memory management (row 8) → accept
syscalls and schedule tasks (rows 9–11) → everything after that is
ordinary kernel logic that rarely needs to drop back into raw asm at
all, which is the entire point of doing this work up front: get these
~15 things right once, in a handful of files, and the rest of an OS
(filesystems, drivers, a GUI — see the breadth of what
[OxideOS](../oxideos/00-overview.md) builds on top of exactly this
foundation) is safe Rust the whole way.
