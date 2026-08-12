# 4. Hello, World — Linux ARM64

Same operating system as [Chapter 3](./03-hello-world-linux-x86-64.md),
same job, completely different-looking assembly — because the CPU
architecture underneath is entirely different. Verified working in the
`asm-arm64` Docker container:

```asm
.global _start
.section .text
_start:
    mov x8, #64          // syscall number 64 = write
    mov x0, #1            // 1st arg: file descriptor 1 = stdout
    adr x1, msg           // 2nd arg: address of the text to print
    mov x2, #14            // 3rd arg: how many bytes to print
    svc #0

    mov x8, #93            // syscall number 93 = exit
    mov x0, #0
    svc #0

.section .data
msg:
    .ascii "Hello, world!\n"
```

```bash
docker exec asm-arm64 bash -c "cd /work && as hello_arm.s -o hello_arm.o && ld hello_arm.o -o hello_arm && ./hello_arm"
```

```
Hello, world!
```

## What's identical to Chapter 3, conceptually

The *shape* of the program is exactly the same: set up registers with a
syscall number and its arguments, trap into the kernel, do it twice (once
for `write`, once for `exit`). If you understood Chapter 3, you already
understand the plan here — only the vocabulary for expressing it changed.

## What's genuinely different, and why

**Register names**: `x0`–`x8` instead of `rax`/`rdi`/`rsi`/`rdx`. ARM64
has 31 general-purpose registers, simply named `x0` through `x30` — no
special historical names like x86-64's `rax`
("**a**ccumulator e**x**tended", a name going back to the 1970s) carries.
[Chapter 6](./06-registers-plain-english.md) covers the full register set;
for now, just note there's no AT&T-vs-Intel style choice here — ARM64 has
one standard syntax, used as-is.

**No `$` on immediates, no `%` on registers**: `mov x0, #1` — the `#`
marks an immediate (ARM64's equivalent of x86's `$`), and registers need
no prefix at all. Also notice the argument order is now **destination,
then source** (`mov x8, #64` = "put 64 into x8") — the reverse of AT&T's
source-then-destination. This isn't a second AT&T/Intel-style choice to
make; ARM64 assembly only comes one way.

**Different syscall numbers entirely**: 64 for `write`, 93 for `exit` —
totally unrelated to x86-64's 1 and 60. Syscall numbers are an
OS-and-architecture-specific table with no cross-architecture
relationship; [Chapter 12](./12-syscalls-deep-dive.md) has the fuller
picture, including why Linux even maintains separate number tables per
architecture at all.

**Syscall number goes in `x8`, not `x0`**: on ARM64 Linux, the arguments
themselves start at `x0` (fd, then address, then length occupy `x0`,
`x1`, `x2`) — but the syscall *number* lives in a register that's
*outside* that argument sequence, `x8`. Contrast x86-64, where the number
shares the same register (`rax`) that also carries the *return value*
after the call — a genuinely different convention, not just a renumbering.

**`svc #0` instead of `syscall`**: `svc` stands for "supervisor call" —
same job as x86-64's `syscall` instruction (trap into the kernel), just a
different instruction, because these are two unrelated instruction sets
that happened to converge on the same *idea*. The `#0` is technically an
immediate operand the kernel can inspect, though Linux doesn't use it for
anything — it's there because the instruction's encoding requires some
immediate value, and 0 is the convention.

**`adr x1, msg` instead of `lea msg(%rip), %rsi`**: `adr` computes a
PC-relative address the same way `lea ...(%rip)` did on x86-64 — "the
address of `msg`, relative to where we are right now." It's simpler here
only because `msg` is close enough in memory to `_start` for `adr`'s
limited range (roughly ±1MB) to reach directly. This *will* fail once your
data lives further away — [Chapter 8](./08-memory-and-addressing.md)
covers `adrp`/`add` with `:lo12:`, the two-instruction pattern ARM64 uses
for addresses `adr` can't reach directly, and exactly why a single 4-byte
ARM64 instruction can't just embed an arbitrary 64-bit address the way
x86-64's variable-length instructions sometimes can.

## Side-by-side, so the shape of the difference is visible at a glance

| Step | x86-64 Linux | ARM64 Linux |
|---|---|---|
| syscall number register | `rax` | `x8` |
| arg 1 (fd) | `rdi` | `x0` |
| arg 2 (address) | `rsi` | `x1` |
| arg 3 (length) | `rdx` | `x2` |
| trap instruction | `syscall` | `svc #0` |
| `write` syscall # | 1 | 64 |
| `exit` syscall # | 60 | 93 |
| load an address | `lea msg(%rip), %reg` | `adr reg, msg` |

## Try it yourself

Same exercise as Chapter 3: change the exit code and confirm it with
`echo $?`.

```bash
# in hello_arm.s, replace `mov x0, #0` (the second one) with:
mov x0, #42
```

```bash
docker exec asm-arm64 bash -c "cd /work && as hello_arm.s -o hello_arm.o && ld hello_arm.o -o hello_arm && ./hello_arm; echo \$?"
```

Next: same CPU architecture again, but a different **operating system** —
watch nearly everything about the syscall convention change while the
*instructions themselves* barely change at all —
[Chapter 5](./05-hello-world-macos.md).
