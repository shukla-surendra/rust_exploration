# 7. Instructions 101

The verbs. Every program in this section, no matter how it eventually
gets used, is built from a small set of instruction *categories*: move a
value, do arithmetic, compare two values, and (next chapter) jump based on
that comparison. This chapter covers the first three.

## Moving values

x86-64 (AT&T syntax — **source, then destination**, always):

```asm
mov $5, %rax        # rax = 5              (immediate -> register)
mov %rax, %rbx       # rbx = rax             (register -> register)
mov (%rbx), %rcx      # rcx = *rbx  (the value AT the address in rbx)
mov %rcx, (%rbx)       # *rbx = rcx  (write rcx to the address in rbx)
```

ARM64 — **destination, then source**, the reverse order:

```asm
mov x0, #5           // x0 = 5
mov x1, x0             // x1 = x0
ldr x2, [x1]             // x2 = *x1  (load from the address in x1)
str x2, [x1]              // *x1 = x2  (store to the address in x1)
```

**The difference that matters most here, and it's not the argument
order** — it's what `mov` is even *allowed* to touch. On x86-64, `mov`
(and most other instructions) can read or write memory directly, as
`mov (%rbx), %rcx` above shows. On ARM64, plain `mov` can **only** move
between registers or load an immediate — reaching into memory is always a
*separate* instruction, `ldr` (load register) to read, `str` (store
register) to write. This is the RISC-vs-CISC split from
[Chapter 1](./01-what-is-a-cpu.md) showing up concretely: ARM64 keeps
"touch memory" and "compute something" as strictly separate steps; x86-64
lets you fuse them into one instruction.

Square brackets `[x1]` on ARM64 and parentheses `(%rbx)` on x86-64 mean
the same thing: "not this register's value itself, but the value stored
*at the address this register holds*" — this is called **dereferencing**
a pointer, and if that word is familiar from Rust or C, this is the exact
same operation those languages' `*ptr` syntax compiles down to.

## Arithmetic

x86-64 — the destination is also always one of the two operands (a
**two-operand** form: `add src, dst` really means `dst = dst + src`):

```asm
add %rbx, %rax    # rax = rax + rbx
sub %rbx, %rax    # rax = rax - rbx
inc %rax           # rax = rax + 1
dec %rax            # rax = rax - 1
imul %rbx, %rax       # rax = rax * rbx   (signed multiply)
```

`mul`/`div` (unsigned) and `idiv` (signed divide) are pickier — they
implicitly use `rax`/`rdx` together as a combined 128-bit value, which is
a wrinkle you'll only meet if you go looking for it; nothing in this
section needs it.

ARM64 — a genuinely different, **three-operand** form: destination and
the two sources are all named separately, so nothing gets silently
overwritten mid-calculation:

```asm
add x2, x0, x1     // x2 = x0 + x1     (x0 and x1 both survive, untouched)
sub x2, x0, x1      // x2 = x0 - x1
mul x2, x0, x1       // x2 = x0 * x1
sdiv x2, x0, x1        // x2 = x0 / x1   (signed divide)
```

This three-operand shape is a real, practical advantage RISC gets from
having simple, uniform instruction encoding — and it's why ARM64 code
often needs *fewer* register-shuffling instructions than x86-64 to compute
the same expression, even though ARM64 needs *more* instructions overall
for anything touching memory (since load/store are always separate, as
above).

## Comparison, and the flag that quietly drives every branch

```asm
cmp $10, %rax     # x86-64: compare rax against 10
cmp x0, #10        // ARM64: compare x0 against 10
```

Neither of these does anything you can see directly — no register changes.
What actually happens: the CPU computes `rax - 10` (or `x0 - 10`)
internally, throws the numeric result away, and keeps only *facts about
that result* in a hidden set of bits called **flags** (`EFLAGS` on
x86-64, `NZCV` — Negative, Zero, Carry, oVerflow — on ARM64): was the
result zero (meaning the two values were equal)? Negative? Did it
overflow? [Chapter 9](./09-control-flow.md)'s conditional jumps
(`je`/`jne`/`jl` on x86-64, `b.eq`/`b.ne`/`b.lt` on ARM64) are really just
"jump if this particular flag bit is set" — `cmp` and the branch that
follows it are always a matched pair, and this is exactly why they're
covered together next rather than here.

## A worked example: the exact instructions behind "sum 1 to 10"

This program was built, run, and verified on both Linux x86-64 and Linux
ARM64 (identical result, `55`, confirmed via exit code on both) — full
control-flow explanation is [Chapter 9](./09-control-flow.md)'s job, but
the instructions inside the loop are pure Chapter 7 material, worth
seeing end to end once:

```asm
# x86-64
mov $0, %rbx       # sum = 0
mov $1, %rcx        # i = 1
loop_top:
    add %rcx, %rbx    # sum += i
    inc %rcx
    cmp $11, %rcx
    jl loop_top
```

```asm
// ARM64
mov x1, #0        // sum = 0
mov x2, #1         // i = 1
loop_top:
    add x1, x1, x2    // sum += i
    add x2, x2, #1
    cmp x2, #11
    b.lt loop_top
```

Notice ARM64 has no `inc`/`dec` shorthand — `add x2, x2, #1` does the same
job `inc %rcx` does on x86-64. That's a real, if minor, x86-64 convenience
ARM64's smaller, more regular instruction set simply doesn't bother
providing; one `add` instruction covers every case a dedicated
increment instruction would.

## What's next

[Chapter 8](./08-memory-and-addressing.md) goes deeper on the addressing
forms briefly shown here (`(%rbx)`, `[x1]`, and the more elaborate indexed
forms like `(%rsi,%rcx,8)` from array access) — then
[Chapter 9](./09-control-flow.md) turns `cmp` and the flags register into
actual loops and `if`-style branching.
