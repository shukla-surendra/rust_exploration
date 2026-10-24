# 11. Talking to Hardware: Port-Mapped I/O vs. Memory-Mapped I/O

Chapters 1–6 covered how the CPU addresses *its own* memory (RAM) and
*storage* (disks). This chapter answers a related but different
question: how does the CPU talk to everything else — a UART, a disk
controller, a keyboard, a network card? Those are separate physical
chips, each with a small set of internal registers, and neither "read
from RAM" nor "read from a disk sector" is the mechanism that reaches
them.

## In Plain English

Picture a CPU as an office worker with two completely different ways to
get something done. One way: pick up an internal phone with its own
private line and dial an extension number — that line doesn't connect
to anything except the specific machines wired to it. The other way:
walk over to a labeled shelf and pick up (or put down) a folder — except
some of those "shelves" aren't storage at all, they're slots that
trigger a real machine to do something the instant you touch them. The
private phone line is **port-mapped I/O**. The labeled-shelf-that's-
secretly-a-machine-control is **memory-mapped I/O**. Both get you to
the same place — a real device, doing a real thing — by two genuinely
different routes.

## The problem, precisely

Every ordinary instruction your program runs reads or writes either a
register or RAM. A UART chip, a disk controller, a keyboard controller
— these are physically separate silicon, each exposing a handful of
internal registers (a data register, a status register, a control
register — bytes, not gigabytes). The CPU needs *some* mechanism to
reach those specific registers instead of RAM, and there are exactly
two that exist in real hardware.

## Port-Mapped I/O (PMIO): a second, separate address space

Some CPU architectures — x86 is the enduring example — have a **second
address space** entirely, reached only by dedicated instructions
(`in`/`out` on x86), completely separate from the normal memory address
space. Port number `0x3F8` (the classic PC serial port, COM1) has
**nothing to do with** memory address `0x3F8` — they don't overlap,
don't alias, and an ordinary memory read can never accidentally land on
a port. x86's port space is 64 KB of these numbered "extensions," and
reaching them is a genuinely different kind of instruction from a
memory load — one your program can't even execute unless it's running
at the CPU's highest privilege level (Ring 0 under an OS, [Chapters
3](../asm/03-privilege-levels-and-mode-transitions.md) and this
book's asm section cover exactly why).

## Memory-Mapped I/O (MMIO): reusing the same address space

The other approach needs no special instructions at all: a device's
registers are simply **assigned addresses inside the normal memory
map**, and you reach them with the exact same load/store instructions
you'd use for RAM. Address `0x09000000` might be a UART's data register
on one machine, or perfectly ordinary RAM on another — which one it is
depends entirely on that specific machine's hardware layout, not on
anything visible in the instruction itself.

## Did you understand correctly? Yes — with one important refinement

**Are these the two options for a CPU to communicate with hardware
registers?** Yes, precisely — PMIO and MMIO are exhaustively the two
mechanisms by which a CPU *initiates* a read or write to a device's own
registers. There's no third way to do that specific thing.

**The refinement**: that's the answer to "how does the CPU **read/write
a device's registers**," which is narrower than "how does hardware
communication work" as a whole. Two more pieces complete the full
picture, and neither is a competing alternative to PMIO/MMIO — both are
built *using* them:

- **Interrupts are the reverse direction.** PMIO and MMIO are both
  *CPU-initiated* — the CPU decides when to ask. An **interrupt** is the
  opposite: the *device* signals the CPU, asynchronously, whenever it
  has something ready — "the disk read finished," "a key was pressed" —
  without the CPU sitting in a loop asking over and over (**polling**,
  which is what both mechanisms above look like in their simplest form).
  A real driver almost always uses PMIO or MMIO to *set up* a device,
  then relies on an interrupt to find out when it's done, rather than
  polling for anything slower than a few instructions' worth of wait.
- **DMA (Direct Memory Access) moves bulk data without the CPU relaying
  every byte.** For something like a disk read, having the CPU copy
  each individual byte through a register (that's exactly what the
  ATA/PIO example below does) wastes CPU time on pure data shuffling.
  With DMA, the CPU uses PMIO or MMIO once, briefly, to tell the device
  "write N bytes starting at RAM address X" — and the device then writes
  directly into RAM itself, with no further CPU involvement until it
  raises an interrupt saying it's done.

So the complete picture is: **PMIO/MMIO** (how the CPU reaches a
device's control/status/data registers) + **interrupts** (how a device
gets the CPU's attention without polling) + **DMA** (how bulk data
moves without the CPU copying every byte). Your understanding of the
first piece was exactly right — it's just one third of the full story,
and the other two both depend on it rather than replacing it.

## Neither mechanism is "just like RAM," even MMIO

This is the detail that trips people up the first time: MMIO *looks*
like an ordinary memory access in your code, but treating it like one
is wrong in a way that causes real, confusing bugs:

- **A write that's never read back can be optimized/reordered away for
  RAM — never for a device register.** Writing to a control register has
  a real external effect (enabling a UART, starting a transfer)
  regardless of whether your program ever reads that address again. Real
  device memory has to be marked specially (as **Device memory**, not
  **Normal memory**, in the page tables — [Chapter 5 of the asm
  section](../asm/05-paging-and-mmu-setup.md) names the exact register,
  `MAIR_EL1` on ARM64, that encodes this) specifically so the CPU
  doesn't apply RAM's normal caching/reordering rules to it.
- **The order of two register writes can be load-bearing.** Two ordinary
  RAM writes can sometimes be safely reordered or merged without changing
  what the program observes. Two device-register writes very often
  can't — writing a baud-rate divisor register before vs. after enabling
  a UART produces a genuinely different, wrong result, not just a
  performance difference.

## Seeing both mechanisms on real hardware

The companion assembly tutorial has actual, tested code for both —
[`asm/11-io-ports-and-mmio.md`](../asm/11-io-ports-and-mmio.md) walks
through a real UART talked to via `in`/`out` (x86 PMIO) and the same
UART talked to via `ldr`/`str` to a fixed address (ARM64 MMIO), both
booted directly in QEMU with no OS underneath — port I/O specifically
can't be demonstrated as an ordinary userspace program at all, since
it's blocked outright below Ring 0. That chapter also includes a real
debugging story worth reading once you're comfortable with the
concepts here: a genuinely confusing bare-metal hang, an initial
wrong diagnosis that looked completely plausible, and how comparing
against this repo's real, independently-working
[`hello-kernel`](./07-hello-kernel-overview.md) project (not a made-up
example — an actual sibling project on this machine) exposed the
mistake.

This repo's own OxideOS documentation describes a real disk driver
using exactly the PMIO pattern from this chapter — I/O ports
`0x1F0`–`0x1F7`, polling a status bit before every data transfer, a
documented `IDENTIFY` command as the required init step — see
[`oxideos/oxide_cocepts/06_storage_stack.md`](../oxideos/oxide_cocepts/06_storage_stack.md#the-ataainit-process-detecting-and-identifying-a-disk).
That driver is PIO (Programmed I/O — the CPU relays every byte through
a port, no DMA involved), which is exactly why its own documentation
calls out DMA as the faster alternative it deliberately doesn't use,
for simplicity.

## Quick self-check

- Why can't port `0x3F8` and memory address `0x3F8` ever collide on a
  PMIO architecture — what makes them genuinely separate spaces rather
  than just a naming convention?
- Why does ARM64 have no equivalent of `in`/`out` at all — what does
  that imply about how *every* ARM64 device driver reaches hardware,
  with no exception?
- Interrupts and DMA were both introduced as *not* being a third
  competing mechanism alongside PMIO/MMIO. Explain concretely how a
  real disk-read-via-DMA still depends on PMIO or MMIO at some point in
  the sequence.
- Why is writing to a device's control register still meaningful even
  if no code ever reads that address again — what's different here from
  an ordinary RAM write a compiler might optimize away?

## What's next

This chapter, like 1–6, is a general concept usable on its own. The
[`hello-kernel` case study (Chapters 7–10)](./07-hello-kernel-overview.md)
is where MMIO from this chapter shows up in real, working code — its
UART output is exactly the PL011 MMIO pattern described here, with
nothing hidden.
