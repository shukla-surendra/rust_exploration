# 12. Syscalls, Deep Dive

Chapters 3 through 5 used `write` and `exit` without much ceremony. This
chapter pulls the whole mechanism apart properly: what a syscall actually
*is*, why the numbers differ per OS, and a few more syscalls beyond the
two you've already used.

## Why syscalls exist at all: the privilege boundary

Your program does not run with unrestricted access to the hardware — the
CPU itself enforces this, via **privilege levels** (rings on x86-64,
exception levels on ARM64; the full mechanics are in the
[OS-development section, Chapter 3](../asm/03-privilege-levels-and-mode-transitions.md)).
Ordinary programs run at the least-privileged level; only the kernel runs
at the most-privileged one. Things like "write to this file," "read from
this network socket," or even "print to the screen" ultimately require
touching hardware or shared, security-sensitive state — and the CPU
simply refuses to let unprivileged code do that directly.

A syscall is the sanctioned door between the two: a special instruction
(`syscall` on x86-64, `svc` on ARM64) that doesn't just jump — it
triggers a controlled, hardware-enforced switch *up* to kernel privilege,
at one of a small number of entry points the kernel itself defined in
advance (so your program can't just jump into the *middle* of the kernel
and do anything it wants). The kernel does the privileged work on your
behalf, then switches privilege back down and returns control to you.
This round trip is also why syscalls are meaningfully slower than a
regular function call — the CPU is doing real, hardware-level security
enforcement, not just moving a return address around.

## The convention, restated in full: number in, arguments in, trap

Every syscall follows the same shape you've already used: put a number
identifying *which* syscall in one fixed register, put its arguments in
other fixed registers (in a fixed order), execute the trap instruction,
and (for most syscalls) read a result back out of a register afterward.

| | Linux x86-64 | Linux ARM64 | macOS (both architectures) |
|---|---|---|---|
| Trap instruction | `syscall` | `svc #0` | `syscall` (x86-64) / `svc #0x80` (ARM64) |
| Number register | `rax` | `x8` | `rax` (x86-64) / `x16` (ARM64) |
| Args 1–4 | `rdi`, `rsi`, `rdx`, `r10` | `x0`–`x3` | `rdi`,`rsi`,`rdx`,`r10` (x86-64) / `x0`–`x3` (ARM64) |
| Return value | `rax` | `x0` | `rax` (x86-64) / `x0` (ARM64) |

One detail not yet mentioned: **x86-64's 4th syscall argument goes in
`r10`, not `rcx`**, even though `rcx` is the 4th argument register for an
*ordinary function call* per [Chapter 10](./10-functions-and-the-stack.md)'s
table. This is because the `syscall` instruction itself clobbers `rcx`
internally (it uses it to save the return address, as part of how the
instruction is implemented in hardware) — so the kernel's syscall
convention deliberately routes around it, using `r10` instead purely for
that argument slot. It's a small, easy-to-forget wrinkle exactly because
it only shows up once you're at four-or-more-argument syscalls.

## Why the *numbers themselves* differ, concretely

Each OS maintains its own **syscall table** — literally an array, inside
the kernel's source code, mapping small integers to kernel functions. Two
different kernels (Linux, XNU/Darwin) were written by entirely different
people, over different timelines, and simply assigned numbers to
`write`/`exit`/`open`/etc. in whatever order made sense to each project —
there was never a cross-OS standards body coordinating this. Linux
additionally maintains a **separate table per CPU architecture**
(x86-64's table and ARM64's table assign different numbers to the same
operation, as Chapters 3–4 already showed with `write` = 1 vs. 64) because
each architecture's table was populated independently as Linux was ported
to it, not kept in lockstep by design.

You can see Linux's actual x86-64 table for yourself:

```bash
docker exec asm-amd64 bash -c "grep -A2 '__NR_write\b' /usr/include/asm-generic/unistd.h /usr/include/x86_64-linux-gnu/asm/unistd_64.h 2>/dev/null"
```

## A few more syscalls, all following the exact same pattern

**`read`** — the mirror image of `write`: same argument shape (fd,
buffer address, max bytes), but the buffer gets *filled in* by the kernel,
and the return value is how many bytes were actually read (which can be
less than you asked for — always check it, never assume a `read` filled
the whole buffer). Linux x86-64 number: 0.

```asm
mov $0, %rax           # read
mov $0, %rdi            # fd 0 = stdin
lea buf(%rip), %rsi       # where to put the bytes
mov $64, %rdx               # max bytes to read
syscall
# rax now holds how many bytes were actually read
```

**`open`** — Linux x86-64 number 2 (macOS BSD number 5, ARM64 Linux
number 56) — takes a pointer to a null-terminated filename, flags (e.g.
`O_RDONLY = 0`), and a mode, returning a new file descriptor (or a
negative number on error — syscalls report failure by returning a small
negative value rather than throwing an exception, which
[Chapter 13](./13-arrays-strings-algorithms.md) touches on when checking
results).

**`mmap`** — how a program actually gets more memory at runtime (what
`malloc` is built on, one layer up) — genuinely more involved (six
arguments) and out of scope for hand-writing here, but worth knowing by
name: it's the syscall behind every heap allocation you've ever triggered
indirectly through a higher-level language.

## Checking a syscall's result

`write`'s return value (bytes actually written) and `read`'s (bytes
actually read) live in the same register the syscall number went in
(`rax`/`x0`) — the convention reuses that register for the result, exactly
like an ordinary function call's return value does. A negative result
(interpreted as signed) conventionally means an error, with the specific
negative value identifying *which* error — real production code always
checks this; every example in this section so far skipped that check only
because `write`ing a hard-coded string to a valid, already-open stdout
essentially never fails in practice.

## What's next

You now have the full syscall picture on both OSes. Time to build
something a bit more substantial than "print a fixed string" —
[Chapter 13](./13-arrays-strings-algorithms.md) works through arrays,
strings, and a couple of small classic algorithms, all in the assembly
you now have the full vocabulary for.
