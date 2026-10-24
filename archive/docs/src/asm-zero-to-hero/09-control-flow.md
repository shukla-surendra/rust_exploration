# 9. Control Flow: Loops & Branches

Every `if`, `for`, `while`, and `match` you've ever written in a
higher-level language compiles down to exactly one mechanism: a
conditional jump. This chapter builds that up from `cmp`
([Chapter 7](./07-instructions-101.md)) to a complete, real loop —
verified to run and produce the correct answer on x86-64 Linux, ARM64
Linux, and ARM64 macOS.

## The unconditional jump

The simplest case — just go somewhere else, no condition:

```asm
jmp loop_top      # x86-64
b loop_top          // ARM64
```

## Conditional jumps: acting on the flags `cmp` set

Recall from [Chapter 7](./07-instructions-101.md): `cmp a, b` computes
`a - b` internally and keeps only *facts* about the result (zero?
negative? overflowed?) in the hidden flags register. A conditional jump
reads those flags and decides whether to actually jump:

| Meaning | x86-64 (after `cmp $b, $a`) | ARM64 (after `cmp x_a, x_b`) |
|---|---|---|
| jump if equal | `je` | `b.eq` |
| jump if not equal | `jne` | `b.ne` |
| jump if less than (signed) | `jl` | `b.lt` |
| jump if less or equal (signed) | `jle` | `b.le` |
| jump if greater than (signed) | `jg` | `b.gt` |
| jump if greater or equal (signed) | `jge` | `b.ge` |
| jump if below (unsigned) | `jb` | `b.lo` |
| jump if above (unsigned) | `ja` | `b.hi` |

**Signed vs. unsigned matters and is easy to get wrong**: the *same bit
pattern* can mean a large positive number or a negative one depending on
which comparison you use — `jl`/`b.lt` treat the top bit as a sign, while
`jb`/`b.lo` treat every bit as pure magnitude. Comparing two loop counters
that can never go negative, unsigned comparisons are the theoretically
"correct" choice, though this section uses the signed forms throughout
since every value involved is small and non-negative either way, and the
signed mnemonics are more likely what you'll recognize from a debugger's
default disassembly.

## Building a loop from these two pieces

The general shape, in either language, is always: a label marking the top
of the loop, the work you want repeated, a `cmp` against the stopping
condition, and a conditional jump back to the top:

```
loop_top:
    <do the work>
    <update the loop variable>
    cmp <loop variable>, <limit>
    j<condition> loop_top     # if not yet done, jump back
```

## Worked example: summing 1 through 10

This exact program was assembled, linked, and run on all three tested
platforms, with the result read straight out of the exit code (`echo
$?`), confirming `55` (1+2+...+10) every time:

```asm
# x86-64 (Linux)
.global _start
.section .text
_start:
    mov $0, %rbx        # sum = 0
    mov $1, %rcx         # i = 1
loop_top:
    add %rcx, %rbx        # sum += i
    inc %rcx
    cmp $11, %rcx
    jl loop_top

    mov %rbx, %rdi         # exit code = sum
    mov $60, %rax
    syscall
```

```asm
// ARM64 (Linux; on macOS, use `_main`/`x16`/`svc #0x80` per Chapter 5)
.global _start
.section .text
_start:
    mov x1, #0          // sum = 0
    mov x2, #1            // i = 1
loop_top:
    add x1, x1, x2         // sum += i
    add x2, x2, #1
    cmp x2, #11
    b.lt loop_top

    mov x0, x1               // exit code = sum
    mov x8, #93
    svc #0
```

```bash
docker exec asm-amd64 bash -c "cd /work && as sum.s -o sum.o && ld sum.o -o sum && ./sum; echo \$?"
# 55
docker exec asm-arm64 bash -c "cd /work && as sum_arm.s -o sum_arm.o && ld sum_arm.o -o sum_arm && ./sum_arm; echo \$?"
# 55
```

**Trace it by hand once, it's worth doing**: `rbx`/`x1` (sum) starts at 0,
`rcx`/`x2` (i) starts at 1. Each pass: add i into sum, increment i,
compare the *new* i against 11, loop again if it's still less than 11.
When `i` reaches 11, the loop exits with `sum = 1+2+...+10 = 55` — the
comparison against `11` rather than `10` is exactly what makes this a
proper "run 10 times" loop instead of 9 or 11; off-by-one mistakes here
are the single most common bug in hand-written loops, in any language.

## `if`/`else`, using the exact same tools

A loop is really just an `if` that jumps *backward*; a plain conditional
is the same mechanism jumping *forward*, past a block of code:

```asm
# x86-64: if (rax > 100) rbx = 1; else rbx = 0;
cmp $100, %rax
jle else_branch
mov $1, %rbx
jmp end_if
else_branch:
mov $0, %rbx
end_if:
```

```asm
// ARM64: same logic
cmp x0, #100
b.le else_branch
mov x1, #1
b end_if
else_branch:
mov x1, #0
end_if:
```

Notice the condition is written as its **negation** (`jle`/`b.le` — jump
to the `else` branch when the *opposite* of your `if` condition holds).
This inverted-condition pattern is exactly what a compiler generates from
`if`/`else` in any higher-level language — seeing it here demystifies a
common surprise in [Chapter 14](./14-reading-compiler-output.md), when a
straightforward `if x > 100` in Rust or C shows up in the compiler's
output as a jump on `<=`.

## What's next

Loops and branches only take you straight through your own code.
[Chapter 10](./10-functions-and-the-stack.md) covers the other half of
control flow — calling into a *different* piece of code and getting back
— which is where x86-64 and ARM64's difference in
[Chapter 6](./06-registers-plain-english.md#the-single-biggest-structural-difference-where-the-return-address-lives)
(stack-based vs. register-based return addresses) stops being an
abstract note and starts actually mattering.
