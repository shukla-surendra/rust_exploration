# 19. Full Instruction Reference: x86-64 and ARM64

[Chapter 16](./16-cheat-sheet.md) is a distillation of only what
Chapters 1–18 actually used. This appendix is the wider net: every
general-purpose integer instruction likely to come up once you start
reading real compiler output, real production assembly, or someone
else's hand-written code — organized by category, both architectures
side by side.

**Scope, stated up front**: this is general-purpose *integer*
instructions only — no SIMD/vector (`xmm`/`ymm`/`zmm` on x86-64, `v0`–`v31`
on ARM64), which [Chapter 18](./18-where-to-go-next.md) already
named as a deliberate, separate topic. Every instruction below was
checked to actually **assemble** with `as` on both toolchains before
being listed — the core ones (`mov`, `add`, `cmp`, the jump family) also
carry the full behavioral verification from earlier chapters; the
additional ones here (conditional select, exclusive-load atomics, bit
tests) are syntax-verified, not run through a full example program the
way Chapters 1–18's material was — a narrower but still real bar, worth
being honest about rather than blurring the two together.

## Data Movement

| Purpose | x86-64 | ARM64 | Notes |
|---|---|---|---|
| Copy a value | `mov %rbx, %rax` | `mov x0, x1` | [Chapter 7](./07-instructions-101.md) |
| Compute an address | `lea 8(%rbx), %rax` | `add x0, x1, #8` | ARM64 has no `lea` — plain arithmetic does the same job, since it never touches memory either |
| Load from memory | `mov (%rbx), %rax` | `ldr x0, [x1]` | x86-64 fuses load+use; ARM64 never does — [Chapter 7](./07-instructions-101.md) |
| Store to memory | `mov %rax, (%rbx)` | `str x0, [x1]` | |
| Load/store a pair at once | — (two `mov`s) | `ldp x0, x1, [sp]` / `stp x0, x1, [sp]` | ARM64-specific; the push/pop idiom — [Chapter 10](./10-functions-and-the-stack.md) |
| Zero-extend a smaller value | `movzbl %al, %ecx` (byte→32-bit) | `mov w0, w1` (writing a `w` reg auto-zero-extends into the `x` reg) | ARM64's 32/64-bit register aliasing makes this implicit rather than a separate instruction |
| Sign-extend a smaller value | `movslq %eax, %rcx` (32→64-bit) | `sxtw x0, w1` | |
| Conditional move (branch-free `if`) | `cmove %rcx, %rax` (and `cmovne`/`cmovl`/`cmovg`/…, one per condition) | `csel x0, x1, x2, eq` (picks `x1` or `x2` based on the condition) | Both let you avoid a real branch for a simple choice — good for avoiding branch-misprediction cost in a hot loop |
| Conditional select variants | — | `csinc` (select-or-increment), `csinv` (select-or-invert), `csneg` (select-or-negate) | ARM64-only — no direct x86-64 equivalent; each fuses the select with one more cheap operation |
| Bitwise NOT into a new register | `mov %rax, %rbx` + `not %rbx` | `mvn x0, x1` | ARM64's `mvn` (move-not) does both steps in one instruction |

## Arithmetic

| Purpose | x86-64 | ARM64 | Notes |
|---|---|---|---|
| Add / subtract | `add %rbx, %rax` | `add x0, x1, x2` | 2-operand vs. 3-operand — [Chapter 7](./07-instructions-101.md) |
| Increment / decrement | `inc %rax` / `dec %rax` | — (`add x0, x0, #1`) | No dedicated ARM64 instruction |
| Negate | `neg %rax` | `neg x0, x1` | |
| Multiply (signed) | `imul %rbx, %rax` | `mul x0, x1, x2` | |
| Multiply-add / multiply-subtract | — (separate `imul` + `add`) | `madd x0, x1, x2, x3` (x0 = x1\*x2 + x3), `msub`, `mneg` | ARM64 fuses these — see [05_gcd_euclidean](../../../asm_examples/05_gcd_euclidean/) for `msub` computing a remainder |
| Divide (unsigned/signed) | `div %rbx` / `idiv %rbx` (quotient→`rax`, remainder→`rdx`, for free) | `udiv x0, x1, x2` / `sdiv x0, x1, x2` (quotient only — no remainder instruction at all) | The single biggest x86-64/ARM64 arithmetic contrast — see [05_gcd_euclidean](../../../asm_examples/05_gcd_euclidean/) for the `msub`-based remainder workaround this forces |
| Add/subtract **with carry** (multi-word arithmetic) | `adc %rbx, %rax` / `sbb %rbx, %rax` | `adc x0, x1, x2` / `sbc x0, x1, x2` | For numbers wider than one register — chain these to add/subtract 128-bit+ values across register pairs |
| Add/subtract, **setting flags** | `add`/`sub` always set flags | `adds x0, x1, x2` / `subs x0, x1, x2` (the plain `add`/`sub` do **not**) | A real ARM64-specific gotcha: plain ARM64 `add`/`sub` are silent on flags — you must use the `s`-suffixed form if a following `cmp`-like decision depends on the result |

## Logical / Bitwise

| Purpose | x86-64 | ARM64 | Notes |
|---|---|---|---|
| AND / OR / XOR | `and %rbx, %rax` / `or %rbx, %rax` / `xor %rbx, %rax` | `and x0, x1, x2` / `orr x0, x1, x2` / `eor x0, x1, x2` | |
| Bitwise NOT | `not %rax` | `mvn x0, x1` | |
| AND-with-inverted-operand | — (`not` + `and`) | `bic x0, x1, x2` (x0 = x1 AND NOT x2) | ARM64-only fused op — "bit clear," useful for masking off specific bits in one instruction |
| Test without storing a result | `test %rax, %rax` | `tst x0, x1` | Both just set flags from an AND, discarding the actual result — the standard "is this zero / are these bits set" idiom |
| Compare a negated value | — | `cmn x0, x1` (compares x0 against -x1) | ARM64-only; equivalent to `cmp x0, -x1` without materializing the negation first |
| Bit test / set / clear | `bt %rbx, %rax` / `bts` / `btr` | — (`and`/`orr`/`bic` with a shifted mask) | x86-64 has dedicated single-bit instructions; ARM64 expresses the same thing with a mask |
| Find first/last set bit | `bsf %rbx, %rax` / `bsr %rbx, %rax` | `clz x0, x1` (count leading zeros — the ARM64-idiomatic version of "find highest set bit") | Different framing of a related question — clz counts from the top, bsr finds the top set bit's index |

## Shifts and Rotates

| Purpose | x86-64 | ARM64 |
|---|---|---|
| Shift left | `shl $1, %rax` (also `sal`, identical) | `lsl x0, x1, #1` |
| Shift right, logical (zero-fill) | `shr $1, %rax` | `lsr x0, x1, #1` |
| Shift right, arithmetic (sign-fill) | `sar $1, %rax` | `asr x0, x1, #1` |
| Rotate | `rol $1, %rax` / `ror $1, %rax` | `ror x0, x1, #1` (left-rotate has no dedicated instruction — expressed as a right-rotate by the complementary amount) |

## Comparison and Condition Codes

Both architectures compute a comparison, then act on hidden flag bits —
[Chapter 7](./07-instructions-101.md) covers the mechanism. The full
condition-code vocabulary, since Chapter 9 only used a handful:

| Meaning | x86-64 suffix | ARM64 suffix |
|---|---|---|
| equal / not equal | `e` / `ne` | `eq` / `ne` |
| less than (signed) | `l` | `lt` |
| less or equal (signed) | `le` | `le` |
| greater than (signed) | `g` | `gt` |
| greater or equal (signed) | `ge` | `ge` |
| below / above (unsigned) | `b` / `a` | `lo` / `hi` |
| below-or-equal / above-or-equal (unsigned) | `be` / `ae` | `ls` / `hs` |
| overflow set / clear | `o` / `no` | `vs` / `vc` |
| negative (sign bit set) / not | `s` / `ns` | `mi` / `pl` |

Append any of these to `j` (x86-64 jump) or `b.` (ARM64 branch),
`cmov` (x86-64 conditional move), or `cs`/`cset` (ARM64 conditional
select family) — e.g. `jle`, `b.le`, `cmovle`, `csel ..., le`.

## Control Flow

| Purpose | x86-64 | ARM64 |
|---|---|---|
| Unconditional jump | `jmp label` | `b label` |
| Conditional jump | `jle label` (see table above) | `b.le label` |
| Compare-and-branch against zero | — (`cmp $0, %rax` + `je`) | `cbz x0, label` / `cbnz x0, label` — fuses the comparison, saving an instruction for this extremely common case |
| Test a single bit and branch | — (`test`/`bt` + `je`) | `tbz x0, #3, label` / `tbnz x0, #3, label` — branch if bit 3 of x0 is zero/nonzero, fused |
| Function call | `call label` | `bl label` |
| Call through a register (function pointer) | `call *%rax` | `blr x0` |
| Return | `ret` | `ret` |
| No-op | `nop` | `nop` |

`cbz`/`cbnz`/`tbz`/`tbnz` are worth calling out specifically: they're
ARM64 folding a comparison and a branch into one instruction for the
single most common cases (against zero, against one bit) — a real,
concrete instance of ARM64 sometimes needing *fewer* instructions than
x86-64 for a specific idiom, the mirror image of load/store always being
separate from arithmetic.

## Stack and Function-Call Support

Already covered in depth in [Chapter 10](./10-functions-and-the-stack.md)
— summarized here for completeness:

| Purpose | x86-64 | ARM64 |
|---|---|---|
| Push / pop | `push %rax` / `pop %rax` | — (`str`/`ldr` with `sp`, or `stp`/`ldp` for a pair) |
| Save a register pair + move the stack pointer, atomically | — | `stp x29, x30, [sp, #-16]!` |
| Return address | pushed to the stack by `call` | held in `x30` (`lr`) by `bl` |

## Atomics and Memory Ordering

A deep dive on this belongs to this book's [OS-development assembly
section](../asm/08-atomics-and-memory-barriers.md) — this is the
lookup table, not the explanation.

| Purpose | x86-64 | ARM64 |
|---|---|---|
| Atomic exchange | `xchg %rax, %rbx` (implicitly locked — no `lock` prefix needed) | `swp x0, x1, [x2]` (ARMv8.1+ LSE) |
| Atomic add | `lock xadd %rbx, (%rax)` | `ldadd x0, x1, [x2]` (ARMv8.1+ LSE) |
| Atomic compare-and-swap | `lock cmpxchg %rbx, (%rax)` | `cas x0, x1, [x2]` (ARMv8.1+ LSE) |
| Exclusive load/store (older, more portable ARM64 form) | — | `ldxr x0, [x1]` + `stxr w2, x0, [x1]` — a manual load-linked/store-conditional pair, the mechanism LSE's `cas`/`ldadd`/`swp` exist to replace with one instruction |
| Load/store with acquire/release ordering | (x86-64's default ordering is already strong enough that plain `mov` usually suffices — see the linked chapter) | `ldaxr` / `stlxr`, or plain `ldar`/`stlr` |
| Memory barrier | `mfence` / `lfence` / `sfence` | `dmb sy` (data memory barrier), `dsb sy` (data synchronization barrier, stronger), `isb` (instruction synchronization barrier) |

**A real, verified gotcha worth keeping**: `lock` only accepts a
**memory** destination operand — `lock xadd %rbx, %rax` (register
destination) fails to assemble outright (`expecting lockable instruction
after 'lock'`); it has to be `lock xadd %rbx, (%rax)`. This makes sense
once stated — `lock` exists to lock a shared memory location for the
duration of the operation, and a register has no such thing as shared
access to lock in the first place — but it's exactly the kind of thing
that only becomes obvious after the assembler rejects it once.

## Miscellaneous / System

| Purpose | x86-64 | ARM64 |
|---|---|---|
| Trap into the kernel | `syscall` | `svc #0` (Linux) / `svc #0x80` (macOS) — [Chapter 12](./12-syscalls-deep-dive.md) |
| Software breakpoint (debugger trap) | `int3` | `brk #0` |
| Read a system/control register | `rdmsr` (privileged) | `mrs x0, <system-reg>` |
| Write a system/control register | `wrmsr` (privileged) | `msr <system-reg>, x0` |
| Port-mapped I/O (read/write a device via a port number, not a memory address) | `in %dx, %al` / `out %al, %dx` (privileged) | — no equivalent; ARM64 uses MMIO exclusively — [OS-dev asm Chapter 11](../asm/11-io-ports-and-mmio.md) |

## What's next

This is the reference to keep open in a tab going forward — nothing
here needs to be memorized ahead of time the way Chapters 1–18 built up
understanding step by step. When you hit an unfamiliar mnemonic in real
disassembly, this is where to look it up; when you're ready to go
further than "look it up," [Chapter 18](./18-where-to-go-next.md)'s
pointers (SIMD, the OS-development section) are still where that path
continues.
