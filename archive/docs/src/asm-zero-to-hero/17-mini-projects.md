# 17. Mini Projects

Three complete, real programs, each combining several earlier chapters at
once. Every one below was assembled, linked, and run for real — read the
notes after each, they call out exactly which earlier chapter each piece
came from, so nothing here should feel unfamiliar.

## Project 1: Recursive factorial (revisited, all three platforms)

Already built in full in [Chapter 10](./10-functions-and-the-stack.md) —
worth revisiting here just to note what it exercises: recursion and the
stack ([Chapter 10](./10-functions-and-the-stack.md)), comparison and
branching ([Chapter 9](./09-control-flow.md)), and the syscall
conventions of all three platforms it was verified on
([Chapters 3–5](./03-hello-world-linux-x86-64.md)) — `factorial(5) = 120`,
confirmed via exit code on Linux x86-64, Linux ARM64, and macOS ARM64.

## Project 2: Iterative Fibonacci

A deliberate contrast with Project 1 — same *kind* of problem (a classic
recursive-looking sequence), solved **iteratively** instead, which needs
no stack bookkeeping at all: just two running values swapped each pass,
exactly the loop pattern from [Chapter 9](./09-control-flow.md). Verified
on Linux x86-64, Linux ARM64, and macOS ARM64 — all three producing
`fib(11) = 89`.

```asm
# x86-64 (Linux) — verified, exit code 89
.global _start
.section .text
_start:
    mov $0, %rax         # a = fib(0)
    mov $1, %rbx           # b = fib(1)
    mov $11, %rcx             # run 11 iterations -> a becomes fib(11)
fib_loop:
    cmp $0, %rcx
    je fib_done
    mov %rax, %rdx              # tmp = a
    mov %rbx, %rax                # a = b
    add %rdx, %rbx                  # b = tmp + b   (i.e. old_a + b)
    dec %rcx
    jmp fib_loop
fib_done:
    mov %rax, %rdi
    mov $60, %rax
    syscall
```

```asm
// ARM64 (Linux; swap the exit sequence per Chapter 5 for macOS) — verified, exit code 89
.global _start
.section .text
_start:
    mov x0, #0          // a = fib(0)
    mov x1, #1            // b = fib(1)
    mov x2, #11              // run 11 iterations
fib_loop:
    cmp x2, #0
    b.eq fib_done
    mov x3, x0                 // tmp = a
    mov x0, x1                   // a = b
    add x1, x3, x1                  // b = tmp + b
    sub x2, x2, #1
    b fib_loop
fib_done:
    mov x8, #93
    svc #0
```

**Why no stack, no `call`/`bl` at all**: unlike Project 1's recursive
factorial, this version never calls a function — it's pure
straight-line-loop arithmetic, three registers doing all the work. This
is a genuinely important, general lesson beyond just assembly: **any
recursive algorithm that only depends on the *immediately previous* one
or two results can usually be rewritten iteratively**, trading the
recursive version's function-call overhead (and, in a real high-level
language, its stack-depth limit) for a fixed, small amount of register
state. Comparing this file to Chapter 10's recursive factorial side by
side is a good way to *feel* that trade-off rather than just read about
it.

## Project 3: Array min/max, printed with `printf`

Combines indexed array access ([Chapter 8](./08-memory-and-addressing.md)/
[Chapter 13](./13-arrays-strings-algorithms.md)), a running comparison
inside a loop ([Chapter 9](./09-control-flow.md)), and a real C library
call ([Chapter 11](./11-calling-c-from-asm.md)) — genuinely the closest
thing in this section to "a small program you might actually write
assembly for." Verified on Linux x86-64:

```asm
.global main
.section .text
main:
    push %rbp
    mov %rsp, %rbp

    lea arr(%rip), %rsi
    mov (%rsi), %rax        # min = arr[0]
    mov (%rsi), %rbx          # max = arr[0]
    mov $1, %rcx                # i = 1
scan_loop:
    cmp $6, %rcx                 # 6 elements in arr
    je scan_done
    mov (%rsi,%rcx,8), %rdx        # rdx = arr[i]
    cmp %rax, %rdx
    jge skip_min                     # if arr[i] >= min, don't update min
    mov %rdx, %rax
skip_min:
    cmp %rbx, %rdx
    jle skip_max                       # if arr[i] <= max, don't update max
    mov %rdx, %rbx
skip_max:
    inc %rcx
    jmp scan_loop
scan_done:
    lea fmt(%rip), %rdi
    mov %rax, %rsi              # arg 2: min
    mov %rbx, %rdx                # arg 3: max
    xor %al, %al                    # no vector-register args (Chapter 11)
    call printf

    xor %eax, %eax
    pop %rbp
    ret

.section .data
arr:
    .quad 42, 7, 99, 3, 56, 21
fmt:
    .asciz "min=%ld max=%ld\n"
```

```bash
docker exec asm-amd64 bash -c "cd /work && gcc minmax.s -o minmax -no-pie && ./minmax"
```

```
min=3 max=99
```

**Reading the core loop once, plainly**: `rax` tracks the running minimum,
`rbx` the running maximum, starting both at `arr[0]`. For each remaining
element, `jge skip_min` reads as "if this element is not smaller than the
current minimum, skip updating it" — the *inverse* of the condition you'd
say out loud ("update min if smaller"), which is exactly the
inverted-branch pattern [Chapter 9](./09-control-flow.md#ifelse-using-the-exact-same-tools)
flagged as what a compiler generates from a plain `if` — seeing it show up
naturally here, in code written by hand rather than by a compiler, is a
good sign the pattern has genuinely sunk in rather than just being a fact
about compilers specifically.

This is also the first program in this whole section combining *three*
different concerns (array traversal, conditional state update, and a
formatted C library call) into one piece — if you can read this one
top-to-bottom without needing to re-check an earlier chapter, that's a
solid, honest signal you're through the "zero" part of zero-to-hero.

## Extend these yourself, before moving on

A few natural next steps, deliberately left as exercises rather than
solved here — each is a small, bounded stretch of what you already have
above:

- **Port Project 3 to ARM64** — the array-scan logic doesn't change at
  all conceptually; only the register names, `ldr`/`cmp`/`b.ge`/`b.le`
  instruction spellings, and (if targeting macOS) the variadic-args-on-
  stack handling from [Chapter 11](./11-calling-c-from-asm.md) do.
- **Make Project 2's iteration count a runtime input** instead of a
  hard-coded `11` — read it via the `read` syscall from
  [Chapter 12](./12-syscalls-deep-dive.md), convert the ASCII digit(s) to
  a number (a short loop: for each digit character, `running_total =
  running_total * 10 + (digit - '0')`), and loop that many times instead.
- **Sum of squares of an array** — take Project 3's scan loop, replace
  the min/max comparison with `imul %rdx, %rdx` (square the current
  element) added into a running total — pure reuse of
  [Chapter 13](./13-arrays-strings-algorithms.md)'s array-sum pattern plus
  [Chapter 7](./07-instructions-101.md)'s multiply instruction.

## What's next

[Chapter 18](./18-where-to-go-next.md) closes out this section: what's
deliberately out of scope here (SIMD/vector instructions), and where to
go next — including a direct hand-off into this book's existing,
narrower [OS-development assembly section](../asm/00-overview.md), which
should now read as a natural continuation rather than a cold start.
