# Assembly, Zero to Hero: x86-64 & ARM64, on macOS & Linux

## Who this is for

Someone who has written code in a "normal" language (Rust, Python, C,
whatever) but has never written a line of assembly and finds the whole
subject intimidating — registers, syscalls, ABIs, two totally different
instruction sets, two totally different operating systems. This section
starts at **true zero**: what a CPU even is. No prior assembly knowledge
assumed anywhere.

By the end you will be able to: read assembly without panicking, write
small real programs in it, understand *why* x86-64 and ARM64 assembly look
so different from each other, understand *why* the exact same architecture
looks different again on Linux vs macOS, use a debugger to watch your code
execute one instruction at a time, and read the assembly your compiler
generates from Rust or C well enough to know what it's doing.

## Why bother, in 2026, when nobody hand-writes assembly anymore

Fair question — almost nobody ships hand-written assembly. The value isn't
"you'll write asm for a living." It's what understanding it unlocks:

- **Debugging stops being magic.** A segfault, a stack overflow, an
  optimizer doing something surprising — these all eventually bottom out
  in registers and memory addresses. Once you can read a disassembly, "the
  program crashed" turns into "the program crashed because register `x1`
  held a null pointer at instruction `0x1004`," which is a problem you can
  actually fix.
- **Every abstraction you use daily has a floor, and this is it.** Rust's
  ownership model, a stack overflow error, why recursion is bounded, why a
  data race is a real hardware phenomenon and not just a language rule —
  all of it is easier to reason about once you've seen the register/stack
  machinery underneath at least once.
- **It explains WHY, not just WHAT.** "Use a `Vec` for cache locality" is
  advice you can follow blindly, or advice you can actually *feel* once
  you've seen that a CPU fetches memory in cache-line-sized chunks and a
  pointer-chasing linked list defeats that on purpose.

## The four-way split this section is built around

Two architectures, two operating systems, four combinations — and every
one of them is genuinely different in ways that matter:

| | Linux | macOS |
|---|---|---|
| **x86-64** (Intel/AMD) | System V AMD64 ABI, ELF binaries, raw `syscall` | Same CPU ISA, but Mach-O binaries, BSD syscall numbers, and (on Apple Silicon) only runs at all via Rosetta 2 translation |
| **ARM64 / aarch64** | Standard AAPCS64 ABI, ELF binaries, raw `svc #0` syscalls, syscall number in `x8` | Apple's own ABI variant (**variadic arguments go on the stack, not in registers** — a real, documented deviation), Mach-O binaries, BSD syscalls via `svc #0x80`, syscall number in `x16`, and requires ad-hoc code-signing to even run |

Every example in this section was actually assembled, linked, and run —
on native Apple Silicon macOS for the two macOS targets, and inside Docker
containers running Ubuntu for the two Linux targets (`docker run
--platform linux/amd64` / `--platform linux/arm64` — the trick used
throughout this section so a single Mac can test all four combinations).
Nothing here is "should work" — every hello-world, loop, function call,
and syscall shown was verified to actually run and produce the stated
output before being written down.

## Roadmap

1. [What Is a CPU, Really?](./01-what-is-a-cpu.md) — zero-jargon
   foundations: fetch-decode-execute, machine code vs. assembly
2. [Setting Up Your Toolchain](./02-toolchain-setup.md) — tools on macOS
   and Linux, plus the Docker trick for cross-testing
3. [Hello, World — Linux x86-64](./03-hello-world-linux-x86-64.md)
4. [Hello, World — Linux ARM64](./04-hello-world-linux-arm64.md)
5. [Hello, World — macOS (both architectures)](./05-hello-world-macos.md)
6. [Registers, in Plain English](./06-registers-plain-english.md)
7. [Instructions 101](./07-instructions-101.md) — `mov`, arithmetic,
   comparison
8. [Memory & Addressing Modes](./08-memory-and-addressing.md)
9. [Control Flow: Loops & Branches](./09-control-flow.md)
10. [Functions & the Stack](./10-functions-and-the-stack.md) — `call`/`ret`
    vs `bl`/`ret`, recursion
11. [Calling C From Assembly (and Back)](./11-calling-c-from-asm.md) —
    including a real Apple ABI gotcha discovered while testing this chapter
12. [Syscalls, Deep Dive](./12-syscalls-deep-dive.md)
13. [Arrays, Strings & Mini-Algorithms](./13-arrays-strings-algorithms.md)
14. [Reading Compiler-Generated Assembly](./14-reading-compiler-output.md) —
    connecting Rust/C source to what actually runs
15. [Debugging With LLDB & GDB](./15-debugging.md)
16. [x86-64 vs ARM64 Cheat Sheet](./16-cheat-sheet.md)
17. [Mini Projects](./17-mini-projects.md) — factorial, Fibonacci, a tiny
    calculator, built for real and run on multiple platforms
18. [Where to Go Next](./18-where-to-go-next.md) — SIMD/NEON, and the
    existing [OS-development-focused assembly section](../asm/00-overview.md)
    in this book
19. [Full Instruction Reference](./19-instruction-reference.md) — every
    general-purpose integer instruction from both architectures, one
    lookup table, organized by category

## How this differs from the other assembly section in this book

This book already has an [`asm/`](../asm/00-overview.md) section — it's
narrower and deeper on purpose: **OS-development assembly only**
(privilege levels, interrupt vector tables, paging, context switches),
assuming you already know general assembly. This section is the opposite
angle: **general-purpose assembly from absolute zero**, for everyday
programs, not kernels. Finish this section, and that one gets a lot less
intimidating — [Chapter 18](./18-where-to-go-next.md) hands off to it
directly.

## Prerequisites

None. Seriously — no assembly, no C, nothing beyond "I've written a loop
and a function before in some language." Every term gets defined the first
time it's used.
