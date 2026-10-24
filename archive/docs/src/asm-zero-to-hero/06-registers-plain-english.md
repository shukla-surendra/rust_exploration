# 6. Registers, in Plain English

## What a register actually is

Picture your desk while working: papers stacked in a filing cabinet
across the room (that's RAM — plenty of space, but takes a moment to walk
over and fetch something), versus the two or three things you're actively
holding in your hands right now (that's registers — vanishingly small
capacity, but zero delay to use).

A register is a small piece of storage **built directly into the CPU
chip itself**, not a memory address. That physical closeness is the whole
point: reading or writing a register happens in a single CPU cycle, while
reading RAM — even fast modern RAM — takes tens to hundreds of cycles by
comparison, because the request has to leave the chip, cross a physical
bus, and come back. Nearly every instruction in [Chapter 7](./07-instructions-101.md)
works on registers precisely because that speed is what makes billions of
instructions per second possible at all.

Every architecture has a fixed, small number of them: x86-64 has 16
general-purpose registers, ARM64 has 31. "Small" is relative to RAM's
gigabytes, but it's deliberate — more registers sounds free, but each one
costs real silicon area and, in every instruction that names a register,
more encoding bits to identify it among a larger set. The register count
is a genuine hardware design trade-off baked into the chip at
manufacture, not a software limit.

## x86-64's registers, and why the names are so historically weird

| Register | Common use in this section | Where the name comes from |
|---|---|---|
| `rax` | syscall number; return value from functions | "**a**ccumulator e**x**tended" |
| `rbx` | general purpose | "**b**ase e**x**tended" |
| `rcx` | general purpose, often loop counters | "**c**ounter e**x**tended" |
| `rdx` | general purpose | "**d**ata e**x**tended" |
| `rsi` | 2nd syscall/function argument | "**s**ource **i**ndex" (originally for string-copy instructions) |
| `rdi` | 1st syscall/function argument | "**d**estination **i**ndex" |
| `rsp` | stack pointer — see [Chapter 10](./10-functions-and-the-stack.md) | "**s**tack **p**ointer" |
| `rbp` | conventional frame/base pointer | "**b**ase **p**ointer" |
| `r8`–`r15` | general purpose, 3rd+ arguments | just numbered — added later, no legacy name |

The `r` prefix and `x` suffix on the first eight are a genuine artifact of
history: the *very same physical register* has been reused and widened
across four decades of x86 evolution — `al` (8-bit, 1978) → `ax` (16-bit)
→ `eax` (32-bit, "**e**xtended `ax`") → `rax` (64-bit, "**r**64 extended
`eax`"). All four names still work today and refer to overlapping pieces
of the *same* physical register — writing to `al` only touches the bottom
8 bits of `rax`, leaving the rest untouched, which is exactly the kind of
thing that causes a confusing bug the first time you hit it. `r8`–`r15`
were added later (with x86-64 itself, in 2003) and never accumulated that
legacy naming — they're just numbers.

`rip` — the **instruction pointer** — deserves a special note: it holds
the address of the *next* instruction to execute, is updated automatically
by the CPU after (almost) every instruction, and — unlike every other
register — you cannot `mov` a value directly into it. The only way to
change it is indirectly, through a jump, call, or return instruction. You
already used it in Chapters 3 and 5, in `lea msg(%rip), %rsi` — computing
an address *relative to* wherever `rip` currently is.

## ARM64's registers — a cleaner, later design

ARM64 was designed decades after x86, from scratch, with none of that
backward-compatibility baggage — which shows immediately in how much more
regular its register naming is:

| Register(s) | Use |
|---|---|
| `x0`–`x7` | 1st through 8th function/syscall arguments, and `x0` doubles as the return value |
| `x8` | indirect result register in the standard AAPCS64 ABI; **also the syscall-number register on Linux** ([Chapter 4](./04-hello-world-linux-arm64.md)) |
| `x9`–`x15` | general purpose, caller-saved ([Chapter 10](./10-functions-and-the-stack.md) explains what that means) |
| `x16`–`x17` | reserved as scratch registers for the linker/PLT; **`x16` is the syscall-number register on macOS** ([Chapter 5](./05-hello-world-macos.md)) |
| `x18` | reserved for platform-specific use (on some OSes, points at per-thread data) — avoid it in your own code |
| `x19`–`x28` | general purpose, callee-saved |
| `x29` | conventional frame pointer, called `fp` |
| `x30` | **link register**, called `lr` — holds the return address after a `bl` (branch-with-link) instruction; the single biggest conceptual difference from x86-64, covered fully in [Chapter 10](./10-functions-and-the-stack.md) |
| `sp` | stack pointer — a genuinely special register on ARM64, not just a numbered `x` register, with hardware-enforced 16-byte alignment on some instructions |
| `pc` | program counter — ARM64's equivalent of `rip`, similarly not directly writable by ordinary instructions |
| `xzr` / `wzr` | the **zero register** — always reads as 0, and writes to it are silently discarded. x86-64 has no equivalent; "zeroing a register" there means `xor reg, reg` instead ([Chapter 3](./03-hello-world-linux-x86-64.md)) |

Every `x0`–`x30` register is 64 bits wide; the *exact same* physical
register is also addressable as its bottom 32 bits under the name `w0`
through `w30` — a much more disciplined version of the same "one register,
multiple names" idea x86-64's `al`/`ax`/`eax`/`rax` chain has, but without
four generations of legacy naming layered on top.

## The single biggest structural difference: where the return address lives

This one deserves to be called out on its own, because it's the register
difference with the most real consequences later in this section
(functions in [Chapter 10](./10-functions-and-the-stack.md), interrupt
handling in the [OS-development section](../asm/04-interrupts-and-exceptions.md)):

- **x86-64**: `call` automatically **pushes** the return address onto the
  stack; `ret` automatically **pops** it back off. A function can freely
  call another function, then a third, then a fourth, with zero extra
  bookkeeping — every `call` manages its own stack slot for its own
  return address, and they nest correctly for free.
- **ARM64**: `bl` (branch-with-link) simply **overwrites `x30`** with the
  return address — a single register, not a stack. Call a *second*
  function before you're done needing the first return address, and
  you've just clobbered it, unless *you* explicitly saved the old `x30`
  to the stack first. This is exactly why, starting in
  [Chapter 10](./10-functions-and-the-stack.md), any ARM64 function that
  itself calls another function needs to open with `stp x29, x30, [sp,
  #-16]!` (save the frame pointer and link register) and close with the
  matching restore — a step x86-64 code never needs, because `call`/`ret`
  already handle it via the stack automatically.

## Registers you'll actually touch vs. ones you won't

This section deliberately doesn't cover every register (x86-64 also has
segment registers, EFLAGS's individual bits, MMX/SSE/AVX vector
registers; ARM64 has system registers like `SCTLR_EL1` that only matter
in kernel code). Those live in the
[OS-development assembly section](../asm/00-overview.md) or
[Chapter 18](./18-where-to-go-next.md)'s SIMD pointer. Everything above is
what every example for the rest of *this* section actually uses.

Next: now that you have names for the storage, [Chapter 7](./07-instructions-101.md)
covers the actual verbs — the instructions that move values between
registers, do arithmetic, and compare them.
