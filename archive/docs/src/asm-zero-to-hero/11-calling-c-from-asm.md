# 11. Calling C From Assembly (and Back)

[Chapter 10](./10-functions-and-the-stack.md)'s calling-convention table
is a promise: any function, in any language, that follows the same
rulebook can call — or be called by — your assembly. This chapter puts
that to the test against a real, ordinary C library function, `printf`.

## Linux x86-64: the straightforward case

```asm
.global main
.section .text
main:
    push %rbp
    mov %rsp, %rbp
    lea fmt(%rip), %rdi   # arg 1: format string
    mov $42, %rsi           # arg 2: the integer to print
    xor %al, %al              # zero variadic float-register count — see below
    call printf

    xor %eax, %eax
    pop %rbp
    ret

.section .data
fmt:
    .asciz "The answer is %d\n"
```

```bash
docker exec asm-amd64 bash -c "cd /work && gcc printf_test.s -o printf_test -no-pie && ./printf_test"
```

```
The answer is 42
```

**Two things new here.** First, this function is named `main`, not
`_start`, and gets linked with plain `gcc file.s -o out` instead of
`as`+`ld` by hand — using `gcc` as the linker driver here means it
automatically links in the full C runtime (`libc`), including the startup
code that itself calls `main` for you, and sets up the process the way
`printf` expects before your code ever runs. Writing a raw `_start` and
manually calling into `libc` is possible but adds ceremony this chapter
doesn't need — the point here is calling C, not replacing C's own
plumbing.

Second, `xor %al, %al` right before the `call`. This is a genuine, easy-to
-miss detail of the System V AMD64 ABI: because `printf` is
**variadic** (it can take a different number of arguments each call, and
doesn't know at compile time how many you're passing or of what type), the
convention requires the caller to report, in register `al` (the bottom 8
bits of `rax`), how many *vector/floating-point* registers were used to
pass arguments — 0 in this case, since `42` went through an integer
register (`rsi`). Skip this and `printf` may misbehave on some
implementations, since it's specified to check `al` before touching any
float-argument registers at all.

## ARM64 Linux: same idea, standard AAPCS64

```asm
.global main
.section .text
main:
    stp x29, x30, [sp, #-16]!
    mov x29, sp

    adrp x0, fmt
    add x0, x0, :lo12:fmt   # arg 1: format string
    mov x1, #42                # arg 2: the integer to print
    bl printf

    mov x0, #0
    ldp x29, x30, [sp], #16
    ret

.section .data
fmt:
    .asciz "The answer is %d\n"
```

```bash
docker exec asm-arm64 bash -c "cd /work && gcc printf_test_arm.s -o printf_test_arm && ./printf_test_arm"
```

```
The answer is 42
```

Nothing unusual here — the integer argument goes straight into `x1`,
exactly like a normal non-variadic call would, and it works exactly as
[Chapter 10](./10-functions-and-the-stack.md)'s table predicts. This
version working cleanly on the first try is precisely what made the next
section's failure worth investigating rather than dismissing as a typo.

## macOS ARM64: the same code, and it silently prints garbage

This is worth walking through exactly as it happened while writing this
chapter, because "the same pattern that worked on Linux ARM64 fails
silently on macOS ARM64" is a genuinely useful lesson about *not* assuming
one platform's ABI generalizes to another that shares the same CPU.

First attempt — the direct analog of the Linux ARM64 version above, just
with the macOS-appropriate entry point and addressing:

```asm
.global _main
.section __TEXT,__text
_main:
    stp x29, x30, [sp, #-16]!
    mov x29, sp

    adrp x0, fmt@PAGE
    add x0, x0, fmt@PAGEOFF
    mov x1, #42
    bl _printf

    mov x0, #0
    ldp x29, x30, [sp], #16
    ret

.section __TEXT,__cstring
fmt:
    .asciz "The answer is %d\n"
```

Ran clean, no crash, no error — and printed `The answer is 1802135696`.
Not 42. Some garbage value, different on different runs.

**The cause: Apple's ARM64 ABI is not standard AAPCS64 for variadic
calls.** Standard AAPCS64 (what Linux ARM64 uses) passes variadic
arguments in registers, same as fixed arguments, exactly as
[Chapter 10](./10-functions-and-the-stack.md)'s table describes. **Apple's
arm64 ABI deviates from this specifically for variadic functions: every
argument after the last *fixed* (non-variadic) parameter must be passed on
the stack, not in registers** — a documented, deliberate difference,
not a bug. `printf`'s only fixed parameter is the format string (`x0`);
everything after it, including our `42`, is variadic and Apple's ABI says
it belongs on the stack. Putting it in `x1` instead meant `printf` went
looking for its integer argument on the stack, found whatever garbage
happened to be sitting there, and printed that.

The fix — confirmed by first checking what Apple's own compiler,
`clang`, generates for the identical C call (`printf("The answer is
%d\n", 42);` compiled with `clang -S -O0`), then matching that exact
stack layout by hand:

```asm
.global _main
.section __TEXT,__text
_main:
    sub sp, sp, #32
    stp x29, x30, [sp, #16]
    add x29, sp, #16

    mov x9, sp          // x9 = address of the vararg storage area (sp + 0)
    mov x8, #42
    str x8, [x9]           // the variadic int argument goes ON THE STACK

    adrp x0, fmt@PAGE
    add x0, x0, fmt@PAGEOFF
    bl _printf

    mov x0, #0
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret

.section __TEXT,__cstring
fmt:
    .asciz "The answer is %d\n"
```

```bash
clang printf_macos.s -o printf_macos
./printf_macos
```

```
The answer is 42
```

## The general lesson, not just this one fix

"ARM64" is not one ABI — it's an instruction set that at least two
different, real-world ABIs (standard AAPCS64 on Linux, Apple's own
variant on macOS/iOS) build on top of, differing specifically at the edges
(here: variadic calls) rather than in the common case. The reliable way to
resolve a mismatch like this isn't to guess harder at the spec — it's
exactly what happened above: **compile the equivalent C code with the
platform's own compiler at `-O0` and read what it actually generated**
([Chapter 14](./14-reading-compiler-output.md) covers this technique in
full as a general debugging tool, well beyond just ABI questions).

## Calling your OWN assembly function from C

The reverse direction — briefly, since it follows directly from
[Chapter 10](./10-functions-and-the-stack.md)'s calling convention table
with no new concepts:

```asm
# double_it.s — a function callable from C
.global double_it
.section .text
double_it:
    lea (%rdi,%rdi), %rax   # rax = rdi + rdi, i.e. 2*n, and it's already the return-value register
    ret
```

```c
// main.c
#include <stdio.h>
extern long double_it(long n);
int main(void) {
    printf("%ld\n", double_it(21));   // 42
    return 0;
}
```

```bash
gcc main.c double_it.s -o combined && ./combined   # 42
```

`extern long double_it(long n);` in the C file is a **declaration, not a
definition** — it tells the C compiler "trust me, this symbol exists
somewhere and follows the standard calling convention," which is enough
for `gcc` to generate a correct `call`, and for the linker to resolve it
against the actual function sitting in `double_it.s`'s object file. This
is the exact mechanism Rust's `extern "C"` and `#[no_mangle]`
(used throughout the
[OS-development assembly section](../asm/01-inline-asm-in-rust.md)) build
on, one layer up.

## What's next

[Chapter 12](./12-syscalls-deep-dive.md) goes back to the lower level —
skipping libc entirely and talking to the kernel directly — with the full
picture of why syscall numbers and conventions differ the way Chapters 3
through 5 already showed piece by piece.
