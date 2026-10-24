# 11. I/O Ports & Memory-Mapped I/O: How the CPU Actually Talks to Hardware

This chapter sits conceptually alongside [Chapter 3](./03-privilege-levels-and-mode-transitions.md)–[5](./05-paging-and-mmu-setup.md) even though it's numbered after
[Multicore & SMP](./10-multicore-and-smp.md) (appended here rather than
renumbering the existing sequence). Every earlier chapter assumed you
already know how to talk to a *device* — an interrupt controller, a
disk, a UART — without ever explaining the actual mechanism. This
chapter is that mechanism, and everything in it was built and tested
against real QEMU-emulated hardware while writing it, including a real
bug that reproduced live — not just described from documentation.

## The problem, precisely

Every instruction you've used so far — `mov`, `add`, `ldr` — talks to
either a register or RAM. Neither of those is how you talk to a UART, a
disk controller, or a keyboard controller. Those are physically separate
chips, each with their own tiny set of internal registers (a handful of
bytes, not gigabytes), and the CPU needs *some* way to read and write
those specific registers instead of RAM. There are exactly two
mechanisms in real hardware, and which one(s) a given CPU architecture
supports is a genuine, structural design difference — not a stylistic
choice:

- **Port-Mapped I/O (PMIO)** — a completely separate, second address
  space, 64 KB on x86, reached only by dedicated instructions.
- **Memory-Mapped I/O (MMIO)** — device registers are assigned addresses
  *inside the normal memory address space* and read/written with
  ordinary load/store instructions.

**x86-64 has both.** **ARM64 has only MMIO — there is no port-I/O
instruction on ARM64 at all.** This is exactly the same kind of
structural fork [Chapter 1's OS-development overview](./00-overview.md)
and the asm-zero-to-hero tutorial keep surfacing between these two
architectures, just at the hardware-communication layer this time.

## Port-Mapped I/O: x86-64's separate address space

x86 has two dedicated instructions, `in` and `out`, that talk to a
*different* 64 KB address space entirely — port numbers, not memory
addresses. Port `0x3F8` (the classic COM1 UART) has nothing to do with
memory address `0x3F8`; they don't overlap, don't alias, and a plain
`mov` can never reach a port no matter what address you give it.

**This is real, tested code** — a complete bare-metal program, assembled
and booted directly in QEMU (`qemu-system-x86_64 -kernel kernel.elf`, no
OS, no bootloader beyond QEMU's own Multiboot loader), that talks to a
real 16550-compatible UART entirely through `in`/`out`:

```asm
# UART init - the real hardware spec every 16550-compatible UART expects
uart_init:
    mov $0x3F9, %dx           # IER (Interrupt Enable Register)
    mov $0x00, %al
    out %al, %dx                # disable UART-generated interrupts - this
                                   # chapter only covers polling, not the
                                   # interrupt-driven alternative (see below)

    mov $0x3FB, %dx            # LCR (Line Control Register)
    mov $0x80, %al
    out %al, %dx                 # set DLAB=1: this temporarily repurposes
                                    # ports 0x3F8/0x3F9 as the baud-rate
                                    # divisor registers instead of their
                                    # normal meaning - a real hardware quirk,
                                    # not an assembly convention

    mov $0x3F8, %dx              # divisor low byte
    mov $0x03, %al                 # 115200 / 3 = 38400 baud
    out %al, %dx

    mov $0x3F9, %dx
    mov $0x00, %al                   # divisor high byte
    out %al, %dx

    mov $0x3FB, %dx                    # LCR again: DLAB=0 now, and set the
    mov $0x03, %al                       # actual frame format - 8 data
    out %al, %dx                           # bits, no parity, 1 stop bit
                                              # ("8N1", the near-universal
                                              # UART default)

    mov $0x3FA, %dx                        # FCR: enable + clear the FIFOs
    mov $0xC7, %al
    out %al, %dx

    mov $0x3FC, %dx                          # MCR: RTS/DSR asserted - real
    mov $0x0B, %al                             # hardware expects this even
    out %al, %dx                                 # though nothing's wired to
    ret                                            # the other end in QEMU
```

**Sending a byte requires a handshake, not just a write:**

```asm
uart_putc:                # al = the byte to send
    push %eax
uart_wait:
    mov $0x3FD, %dx          # LSR (Line Status Register)
    in %dx, %al
    test $0x20, %al            # bit 5 = THRE (Transmit Holding Register
    jz uart_wait                 # Empty) - wait until the UART says it's
    pop %eax                       # actually ready for another byte
    mov $0x3F8, %dx
    out %al, %dx
    ret
```

**Verified output**, booted directly with `qemu-system-x86_64 -kernel
kernel.elf -serial mon:stdio -display none`:

```
Hello from bare metal, via real port I/O!
```

Every `out` above is genuinely privileged — this only works because a
freshly-booted Multiboot kernel runs at CPL 0 (ring 0,
[Chapter 3](./03-privilege-levels-and-mode-transitions.md)). The exact
same `in`/`out` instructions, run from an ordinary Linux userspace
program, fault immediately — which is *why* this had to be tested
bare-metal rather than as an ordinary program the way earlier tutorial
chapters were: port I/O is one of the few things in this whole series
that's structurally impossible to demonstrate from ring 3.

## Memory-Mapped I/O: the only option on ARM64, and increasingly common on x86 too

MMIO needs no special instructions at all — a device's registers are
just given fixed addresses inside the normal memory map, and you `ldr`/
`str` (or `mov`) them exactly like RAM. The mechanism is simpler to
describe; the reasoning about *why* it's the same operation as touching
RAM, yet must behave completely differently, is the actual substance of
this section.

Here's the ARM64 equivalent of the x86 UART code above, targeting
QEMU's `virt` machine, which memory-maps a PL011 UART at a fixed
address:

```asm
.equ UART_BASE, 0x09000000   // fixed by QEMU's virt machine model - the
                                // MMIO equivalent of "port 0x3F8 is just
                                // where COM1 lives" on a PC
.equ UART_FR,   0x18            // Flag Register
.equ UART_DR,   0x00              // Data Register
.equ UART_CR,   0x30                // Control Register
.equ UART_IBRD, 0x24                  // baud divisor, integer part
.equ UART_FBRD, 0x28                    // baud divisor, fractional part
.equ UART_LCRH, 0x2C                      // Line Control Register

uart_init:
    mov x2, #UART_BASE
    mov w3, #0
    str w3, [x2, #UART_CR]      // disable the UART while configuring it
    mov w3, #13
    str w3, [x2, #UART_IBRD]
    mov w3, #1
    str w3, [x2, #UART_FBRD]
    mov w3, #0x70                 // 8 data bits + FIFOs enabled
    str w3, [x2, #UART_LCRH]
    mov w3, #0x301                  // UARTEN | TXE | RXE
    str w3, [x2, #UART_CR]
    ret

uart_putc:                    // w1 = the byte to send
    mov x2, #UART_BASE
uart_wait:
    ldr w3, [x2, #UART_FR]
    tst w3, #(1 << 5)            // TXFF - transmit FIFO full
    b.ne uart_wait                 // same handshake as x86's THRE poll,
    strb w1, [x2, #UART_DR]          // through a load instead of `in`
    ret
```

Notice this is *structurally identical* to the x86 version — a
control-register init, then a poll-before-write handshake on a status
bit. Only the instructions used to reach the registers differ (`ldr`/
`str` to a fixed address vs. `in`/`out` to a port number); the actual
protocol a UART expects is the same regardless of which mechanism the
CPU uses to reach it.

**Status: fully verified.** This exact code boots on QEMU's `aarch64`
`virt` machine (`qemu-system-aarch64 -M virt -cpu cortex-a72 -nographic
-kernel k.elf`) and prints:

```
Hello from bare metal, via real MMIO!
```

That confirmation took two rounds of real debugging to get to — both
kept below, because the wrong turn is more instructive than a clean
first-try success would have been.

## The real bug — and the wrong diagnosis that delayed finding it

The first draft had no `uart_init` call at all — just the `uart_putc`
handshake. It hung, with no output and no crash. Instruction-tracing
the execution (`qemu-system-aarch64 -d in_asm`) showed the CPU stuck in
a tight loop, reading `UARTFR`, testing bit 5 (TXFF), and looping
because it read as set — so the natural-looking conclusion was "the
UART was never enabled, so it reports the transmit FIFO as permanently
full." That diagnosis produced the `uart_init` sequence above, which
**is** a correct implementation of the real PL011 spec. But adding it
didn't fix the hang. Something else was wrong, and the UART-init theory
had been a plausible-looking dead end.

**The actual bug**: `_start` never initialized `sp` at all. It jumped
straight into `bl uart_init`, and the very first thing any called
function does on entry — `uart_putc`'s `stp x0, x30, [sp, #-16]!` — is
*write to the stack*, through whatever garbage value happened to be
sitting in `sp` at CPU reset. That's not a clean crash; it's a store to
an unpredictable address, which can silently corrupt registers, memory,
or the function's own return address, and manifest as almost anything —
including, in this case, execution that still looked like it was
"stuck polling TXFF" when traced, because the corruption happened to
leave the CPU somewhere that resembled the earlier hang. The real
`hello-kernel` project this book's
[Systems, Chapters 7–10](../systems/07-hello-kernel-overview.md) case
study is built from does the identical PL011 polling — **with no
UART init at all** — and works correctly, which is what actually
exposed the misdiagnosis: if a real, working project doesn't need
`uart_init`, that was never the actual fix here. The real fix, confirmed
by testing with `uart_init` removed entirely afterward, was exactly
three instructions at the very top of `_start`:

```asm
adrp x0, stack_top
add x0, x0, :lo12:stack_top
mov sp, x0
```

`uart_init` was left in the final file anyway — it's correct, harmless,
and genuinely how you'd configure a real PL011 rather than rely on
whatever state QEMU's model happens to reset into — but it was never
the actual fix for this bug.

**The general lesson, worth carrying forward past this one UART**: when
bare-metal code hangs or behaves in a way that superficially matches a
plausible hardware-protocol explanation, check the absolute fundamentals
— *is the stack pointer even initialized* — before trusting a diagnosis
that requires reasoning about a specific device's register semantics.
The x86 file in this chapter got this right from the very first draft
(`mov $stack_top, %esp`, the second instruction in `_start`); the ARM64
file dropped that step later and paid for it with a debugging session
that chased the wrong theory first. A wrong-but-plausible diagnosis that
happens to correlate with the symptom is a real, recurring trap — the
fix here was found by comparing against independently-working code, not
by reasoning about the failing code harder.

## Why MMIO needs the compiler/CPU to promise something extra

An MMIO register looks exactly like a RAM address in your source code —
`str w3, [x2, #UART_CR]` is indistinguishable, syntactically, from
storing to an ordinary variable. But treating it like ordinary RAM is
wrong in two specific, dangerous ways real hardware drivers have to
guard against:

- **A write to RAM that's never read back can be optimized away —
  a write to a device register never can be**, even if nothing in the
  program ever reads it back. Writing `0x301` to `UART_CR` has a real,
  external side effect (enabling the UART) regardless of whether any
  later instruction reads that memory location again. A compiler
  optimizing hand-written C for this exact register would need `volatile`
  to be told this explicitly; hand-written assembly doesn't have this
  problem *from the compiler*, but the CPU's own out-of-order execution
  and caching can still reorder or cache a plain load/store the same
  incorrect way unless the memory region is marked appropriately.
- **Two device-register accesses can't be silently reordered or merged**
  the way two ordinary RAM writes sometimes safely can — writing
  `UART_IBRD` then `UART_LCRH` in the wrong order, or having the CPU
  decide to coalesce them, produces a UART configured incorrectly, not
  just a performance difference. This is exactly
  [Chapter 8](./08-atomics-and-memory-barriers.md)'s territory — real
  MMIO device drivers mark the memory region as **Device memory** (via
  `MAIR_EL1` on ARM64, [Chapter 5](./05-paging-and-mmu-setup.md#aarch64-ttbr0_el1ttbr1_el1-tcr_el1-mair_el1-sctlr_el1) already named this exact register) specifically
  to disable caching and reordering for that address range, and
  frequently need explicit barrier instructions (`dmb`/`dsb`, also
  Chapter 8) between register writes the hardware requires to happen in
  a strict, observable order.

## Polling vs. interrupts: the two ways to know a device is ready

Every `uart_wait`-style loop above is **polling** — the CPU repeatedly
asking "are you ready yet?" in a tight loop, burning cycles the entire
time it's waiting. This is fine for a UART transmit (the wait is
typically nanoseconds), and it's exactly what both examples in this
chapter do. It's a genuinely bad strategy for a slow device (a disk, a
network card) where the wait could be milliseconds — that's precisely
why [Chapter 4](./04-interrupts-and-exceptions.md) exists: instead of
the CPU asking in a loop, the device *tells* the CPU when it's ready, by
raising an interrupt, and the CPU is free to do other work in the
meantime. Polling is the mechanism this chapter demonstrates because
it's simpler to show working end to end in a few dozen lines; interrupts
are what real production drivers for anything slower than a UART
actually use.

## Real, non-toy prior art already in this workspace

This exact PMIO pattern — port I/O, a poll-before-act status check, a
documented device-specific init sequence — is not unique to a UART. This
workspace's own OxideOS storage-stack documentation describes a real ATA
disk driver following the identical shape: I/O ports `0x1F0`–`0x1F7`,
polling the `BSY` and `DRQ` bits on a status port before reading data,
sending an `IDENTIFY` command (`0xEC`) as a documented init step before
the drive will answer at all — see [`oxideos/oxide_cocepts/06_storage_stack.md`](../oxideos/oxide_cocepts/06_storage_stack.md#the-ataainit-process-detecting-and-identifying-a-disk).
Once this chapter's UART pattern clicks, that driver reads as the exact
same idea, applied to a different device.

## What's next

The asm-zero-to-hero tutorial's
[instruction reference](../asm-zero-to-hero/19-instruction-reference.md)
lists `in`/`out` alongside every other instruction covered across both
tutorials. Within this OS-development section specifically,
[Chapter 8](./08-atomics-and-memory-barriers.md) is the natural next
stop — the memory-ordering guarantees this chapter's MMIO section only
introduced are its actual subject.
