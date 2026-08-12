# 18. Where to Go Next

## What this section deliberately left out, and why

**SIMD/vector instructions** (SSE/AVX on x86-64, NEON on ARM64) —
instructions that operate on several values at once (e.g. adding four
pairs of numbers in a single instruction, instead of a loop doing one
`add` per pair) — were skipped entirely on purpose. They're a genuinely
separate skill built *on top of* everything in this section (you still
need registers, memory addressing, and loops to use them well), not a
prerequisite for it, and they come with their own large register set
(`xmm`/`ymm`/`zmm` on x86-64, `v0`–`v31` on ARM64) and instruction
vocabulary big enough to deserve its own dedicated treatment rather than
a rushed final chapter here. If you go looking: the concept to search for
is "vectorization," and Rust's `std::simd` (nightly) or the
`std::arch::x86_64`/`std::arch::aarch64` intrinsics modules are the
natural entry point from where this section leaves off.

**Assembler macros, `.if`/`.endif` conditional assembly, and linker
scripts** — real tools for larger assembly projects, left out because
nothing in this section's examples was large enough to need them. Once
you're writing more than a screen or two of hand-written assembly at a
time, they're worth a look.

**Windows** — genuinely a fifth combination this section skipped, since
the Mac-with-Docker workflow in [Chapter 2](./02-toolchain-setup.md)
doesn't extend to it cleanly, and the x86-64 calling convention alone
differs enough (`rcx`,`rdx`,`r8`,`r9` for the first four arguments,
instead of System V's `rdi`,`rsi`,`rdx`,`rcx` — a real, separate ABI
called the **Microsoft x64 calling convention**) to deserve its own
treatment rather than a rushed aside.

## The natural next step in this book: OS-development assembly

This book already has a second, narrower assembly section —
[**Assembly for OS Development**](../asm/00-overview.md) — and it should
now read very differently than it would have before this section. It
explicitly scopes itself to *only* what an OS kernel needs raw assembly
for: privilege levels and mode transitions, interrupt/exception handling,
paging and MMU setup, context switching, atomics and memory barriers,
multicore bring-up. Every one of those chapters leans on exactly the
foundation this section just built:

- Its [registers & calling-conventions chapter](../asm/02-registers-and-calling-conventions.md)
  is a more compact restatement of this section's
  [Chapter 6](./06-registers-plain-english.md) and
  [Chapter 10](./10-functions-and-the-stack.md) — you'll recognize the
  `adrp`/`:lo12:` address-loading idiom immediately from
  [this section's Chapter 8](./08-memory-and-addressing.md).
- Its [interrupts & exceptions chapter](../asm/04-interrupts-and-exceptions.md)
  builds ISR (interrupt service routine) stubs that manually save every
  caller-saved register — a direct, higher-stakes application of the
  caller-saved/callee-saved distinction from
  [this section's Chapter 10](./10-functions-and-the-stack.md#calling-conventions-which-registers-hold-what-by-agreement).
- Its [context-switching chapter](../asm/07-context-switching.md) needs
  the *entire* register set saved per task — literally this section's
  [register cheat sheet](./16-cheat-sheet.md), turned into a struct
  layout.

If OS/kernel development is where your curiosity is headed, that section
is the direct continuation of everything built here.

## Two more directions, if kernels aren't the pull

**Embedded systems** — this book's
[Rust for Embedded Systems section](../embedded/00-overview.md) sits at a
similar depth to the OS-development one, but for microcontrollers instead
of full OS kernels: no MMU, no virtual memory, often no OS underneath your
code at all — see its
[no_std chapter](../embedded/01-no-std-and-the-embedded-toolchain.md) for
where that world starts to diverge from what this section covered.

**Just get faster at reading it** — genuinely, the highest-leverage next
step for most people isn't a new topic at all, it's repetition of
[Chapter 14](./14-reading-compiler-output.md)'s technique: pick a small
Rust or C function you've already written for something else, compile it
with `-S` at both `-O0` and `-O2` (or drop it into Compiler Explorer,
godbolt.org, for the same thing live), and predict what you'll see
*before* looking. Getting that prediction right, consistently, on
functions you didn't write for this purpose, is the actual sign that this
section's material has become fluent rather than just completed.

## One last gut check

Go back to [Chapter 1](./01-what-is-a-cpu.md)'s opening claim — that
understanding assembly turns "the program crashed" into "the program
crashed because register `x1` held a null pointer at instruction
`0x1004`." If that sentence reads as an obviously true, unremarkable fact
now rather than something to take on faith, this section did its job.
