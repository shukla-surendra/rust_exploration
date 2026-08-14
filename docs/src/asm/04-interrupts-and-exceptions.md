# 4. Interrupts & Exceptions: IDT vs Exception Vector Table

## Before the IDT: the real-mode IVT

Every x86 CPU begins here, not at the IDT — **the reset vector**,
`0xFFFF0`, hardwired into the CPU itself; the very first instruction
executed after power-on or reset lives there (inside BIOS ROM, mapped
at the top of the 20-bit real-mode address space). BIOS code running
from that point sets up a much older, simpler table before anything
else: the **Interrupt Vector Table (IVT)**, at physical address
`0x00000`.

- 256 entries, 4 bytes each (2-byte `IP` + 2-byte `CS`) = 1 KB total,
  occupying `0x00000`–`0x003FF`.
- No `lidt`/pointer indirection needed — the table's address is fixed
  at `0x0000:0x0000` by convention, and looking up interrupt `N` is
  just `N × 4` (e.g. `INT 0x10`, video services, → offset `0x40`).
  That's *why* it lives at the very bottom of memory: the math is
  trivial hardware logic, not a design choice with alternatives.
- On power-on the IVT is empty; BIOS fills it during POST with
  handlers for hardware IRQs and software services (`INT 0x10` video,
  `INT 0x13` disk, `INT 0x16` keyboard) — this is how 16-bit real-mode
  code (the boot sector, DOS) talks to hardware without touching
  device registers directly, the real-mode equivalent of a driver API.

The IVT only exists in **16-bit real mode** — the mode every x86 CPU
boots into before anything switches it to protected/long mode. Once
[Chapter 5's boot sector](../systems/05-boot-process-bios-uefi.md) code
(or a Stage 2 loader) makes that switch, the IVT is abandoned entirely
in favor of the IDT below — a different table, a different format,
loaded explicitly via `lidt` instead of assumed at a fixed address.

## x86-64: the IDT, one entry per interrupt number

Once the CPU leaves real mode, interrupt dispatch moves from the
fixed-address IVT above to this software-configured table — same
underlying job, entirely different mechanism. The **IDT** (Interrupt
Descriptor Table) is a 256-entry table; entry
`N` points at the handler for interrupt/exception number `N` (divide
error is 0, page fault is 14, syscall is conventionally 128/`0x80` or
handled via `syscall`/`sysret` instead — see
[Syscall Entry & Exit](./06-syscall-entry-exit.md)). Loading it uses the
`lidt` instruction, once, at boot — see
[OxideOS Concepts, Chapter 2](../oxideos/oxide_cocepts/02_interrupts_and_cpu.md)
and [Study Journal 02](../oxideos/study/02_interrupts.md) for the full
IDT setup and the **PIC** (8259A — see the
[hardware reference](../oxideos/study/hardware/8259A_pic.md)) that
routes physical device IRQs (keyboard, timer) onto specific interrupt
numbers.

**The ISR stub — the actual assembly every entry needs**, because the
CPU only saves a minimal frame (`rip`, `cs`, `rflags`, and `rsp`/`ss` if
a privilege change occurred) before jumping to your handler — everything
else is still whatever the interrupted code was using:

```
isr_stub:
    push rax
    push rcx
    push rdx
    ; ... push every caller-saved register (chapter 2's table) ...
    call generic_interrupt_handler   ; ordinary Rust code, now safe to run
    pop rdx
    pop rcx
    pop rax
    ; ... symmetric pops ...
    iretq                            ; return from interrupt
```

`iretq` (not `ret`) is required — it restores `rip`/`cs`/`rflags` (and
`rsp`/`ss` on a privilege change) from the frame the CPU itself pushed,
which `ret` knows nothing about.

## aarch64: one exception vector table, 16 fixed-size slots

aarch64 has no per-interrupt-number table — instead, **one 2 KB table**
per Exception Level, with exactly **16 entries**, each a fixed **128
bytes** (32 instructions), indexed by *category* rather than cause:

| Slot category | Meaning |
|---|---|
| Synchronous | A trap caused directly by the current instruction — a data abort, an `svc` syscall |
| IRQ | A normal maskable hardware interrupt |
| FIQ | A "fast" interrupt (higher priority category) |
| SError | An asynchronous system error |

...×4, once for each combination of "from the same EL, using SP_EL0"
/ "from the same EL, using SP_ELx" / "from a lower EL, AArch64" /
"from a lower EL, AArch32" — 4 categories × 4 source combinations = 16.
Each 128-byte slot is small enough that real handlers almost always
just branch out immediately to a full handler function.

`hello-kernel` doesn't implement this table (chapter 3 noted it stays
at EL1 the whole time, taking no exceptions at all); OxideOS's own
aarch64 port has exactly one EL1 vector table wired up (per the ARM
status notes referenced in
[Systems, Chapter 7](../systems/07-hello-kernel-overview.md)'s
prerequisites) but "every exception just dumps state and halts — no IRQ
dispatch yet," since aarch64 devices there are polled (virtio-mmio)
rather than interrupt-driven.

**The exception entry stub** is the same fundamental shape as x86's ISR
stub — save every register the interrupted code might have been using,
call into Rust, restore, return — but using `stp`/`ldp` (store/load
*pair* — saves two 64-bit registers in one instruction, since aarch64
has no single-register `push`/`pop` mnemonics) and `eret` instead of
`iretq`:

```
vector_slot:
    stp x0, x1, [sp, #-16]!    ; push x0,x1 (pre-decrement, writeback)
    stp x2, x3, [sp, #-16]!
    ; ... save the rest, plus ELR_EL1/SPSR_EL1 ...
    bl  generic_exception_handler
    ; ... restore in reverse order ...
    ldp x2, x3, [sp], #16
    ldp x0, x1, [sp], #16
    eret
```

## Enabling and disabling interrupts

| | x86-64 | aarch64 |
|---|---|---|
| Disable | `cli` | `msr daifset, #0xf` (or a narrower mask — I/F/A/D bits individually) |
| Enable | `sti` | `msr daifclr, #0xf` |

`DAIF` is aarch64's interrupt-mask register — its name is literally the
four maskable exception categories it controls (Debug, SError, IRQ,
FIQ). Both architectures need this around any critical section that
manipulates shared kernel state from code that could itself be
interrupted — the same "many readers XOR one writer" concern from
[Ownership, Borrowing & Lifetimes](../workbook/02-ownership-borrowing-lifetimes.md#rule-3-many-readers-xor-one-writer--enforced-at-compile-time),
just enforced by hand here instead of by the borrow checker, since an
interrupt handler running mid-update on shared kernel data is exactly
the kind of concurrent-mutation hazard that rule exists to prevent.
