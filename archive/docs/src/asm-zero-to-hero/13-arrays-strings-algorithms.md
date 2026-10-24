# 13. Arrays, Strings & Mini-Algorithms

Everything needed for this chapter already exists in earlier ones — this
is where indexed addressing ([Chapter 8](./08-memory-and-addressing.md))
and loops ([Chapter 9](./09-control-flow.md)) combine into the small,
familiar algorithms you'd otherwise only ever see in a "real" language.
Every example below was assembled, linked, and run for real.

## Arrays: declaring one, and summing it

`.quad` reserves 8-byte (64-bit) values — matching a 64-bit register — one
after another, contiguously, which is the entire definition of an array:
a sequence of same-sized values sitting back-to-back in memory, at
addresses `base`, `base+8`, `base+16`, and so on.

```asm
# x86-64 — verified: sums {10,20,30,40,5}, exit code 105
.global _start
.section .text
_start:
    lea arr(%rip), %rsi
    mov $0, %rcx         # index i = 0
    mov $0, %rbx           # sum = 0
sum_loop:
    cmp $5, %rcx
    je done
    mov (%rsi,%rcx,8), %rax   # rax = arr[i]   (see Chapter 8: base+index*8)
    add %rax, %rbx
    inc %rcx
    jmp sum_loop
done:
    mov %rbx, %rdi
    mov $60, %rax
    syscall

.section .data
arr:
    .quad 10, 20, 30, 40, 5
```

```asm
// ARM64 — verified: identical result, 105
.global _start
.section .text
_start:
    adr x1, arr
    mov x2, #0          // index i = 0
    mov x3, #0            // sum = 0
sum_loop:
    cmp x2, #5
    b.eq done
    ldr x4, [x1, x2, lsl #3]   // x4 = arr[i]
    add x3, x3, x4
    add x2, x2, #1
    b sum_loop
done:
    mov x0, x3
    mov x8, #93
    svc #0

.section .data
arr:
    .quad 10, 20, 30, 40, 5
```

```bash
docker exec asm-amd64 bash -c "cd /work && as arrsum.s -o o && ld o -o arrsum && ./arrsum; echo \$?"    # 105
docker exec asm-arm64 bash -c "cd /work && as arrsum_arm.s -o o && ld o -o arrsum_arm && ./arrsum_arm; echo \$?"  # 105
```

Every idea here is one you've already met: `lea`/`adr` for the array's
base address ([Chapter 8](./08-memory-and-addressing.md#pc-relative)),
`(%rsi,%rcx,8)`/`[x1, x2, lsl #3]` for indexed access into it
([Chapter 8](./08-memory-and-addressing.md#indexed-addressing-how-arrays-actually-work-under-the-hood)),
and a `cmp`+conditional-jump loop counting up to the array's length
([Chapter 9](./09-control-flow.md)). "Arrays" were never a separate
concept the CPU understands — they're this exact pattern, applied by a
programmer (or a compiler on your behalf), every time.

## Strings: what a null-terminated string actually is

`"Hello, world!\n"` in Chapters 3–5 was printed by *hand-counting* its
length (14) and hard-coding that number — workable only because the text
never changes. A **C-style string** solves the general case differently:
instead of tracking a length separately, the string's bytes are simply
followed by one extra byte, `0`, marking "this is where the string ends."
`.asciz` (as opposed to plain `.ascii`) is exactly `.ascii` plus that
trailing zero byte, automatically.

`strlen` — computing that length at runtime by walking forward until you
hit the zero byte — is the single most common "walk a string" pattern
there is, and it's every bit as simple as it sounds:

```asm
# x86-64 — verified: "hello, asm!" is 11 characters, exit code 11
.global _start
.section .text
_start:
    lea msg(%rip), %rsi
    mov $0, %rcx            # length counter = 0
count_loop:
    movb (%rsi,%rcx,1), %al   # al = msg[i]   (1-byte load — a string is bytes, not quads)
    cmp $0, %al
    je done
    inc %rcx
    jmp count_loop
done:
    mov %rcx, %rdi
    mov $60, %rax
    syscall

.section .data
msg:
    .asciz "hello, asm!"
```

```bash
docker exec asm-amd64 bash -c "cd /work && as strlen.s -o o && ld o -o strlen && ./strlen; echo \$?"   # 11
```

Two small but important differences from the array-sum loop above:
**`movb`**, not plain `mov` — the `b` suffix explicitly says "move a
single **b**yte," because a string is an array of 1-byte characters, not
8-byte values, and the assembler needs telling which size you mean
whenever it's ambiguous (`al` — the bottom 8 bits of `rax`, from
[Chapter 6](./06-registers-plain-english.md) — is the natural
single-byte destination to pair it with). And the scale factor in
`(%rsi,%rcx,1)` is `1`, not `8`, because each element (each character) is
exactly 1 byte.

## The "count once, print correctly for any string" upgrade

With `strlen` in hand, [Chapters 3–5](./03-hello-world-linux-x86-64.md)'s
hard-coded `mov $14, %rdx` can finally be replaced with something that
works for *any* string, not just this one 14-character one — run
`strlen`'s loop first, then feed its result straight into `write`'s
length argument:

```asm
.global _start
.section .text
_start:
    lea msg(%rip), %rsi
    mov $0, %rcx
count_loop:
    movb (%rsi,%rcx,1), %al
    cmp $0, %al
    je print_it
    inc %rcx
    jmp count_loop
print_it:
    mov $1, %rax
    mov $1, %rdi
    lea msg(%rip), %rsi     # rsi got clobbered as a loop pointer above — reload it
    mov %rcx, %rdx             # length = whatever strlen's loop counted
    syscall

    mov $60, %rax
    xor %rdi, %rdi
    syscall

.section .data
msg:
    .asciz "This string's length was never hard-coded!\n"
```

This is a genuinely useful pattern, not just a toy: it's the exact shape
every real `print`/`puts` implementation uses under the hood — a `strlen`
pass, then a `write`, chained together — before any of the buffering,
formatting, or Unicode handling a real standard library adds on top.

## String reversal — a classic, using two pointers

The other extremely common string exercise: reverse a string **in
place**, using two indices walking toward each other from opposite ends
— the same "two-pointer" technique that shows up constantly in general
algorithm work, not just assembly:

```asm
# x86-64: reverses `buf` in place using rcx (left index) and rdx (right index)
lea buf(%rip), %rsi
mov $0, %rcx              # left = 0
# ... compute strlen into rdx as above, then:
dec %rdx                    # right = length - 1 (index of the last real character)
reverse_loop:
    cmp %rcx, %rdx
    jle reverse_done          # stop once left meets or passes right
    movb (%rsi,%rcx,1), %al     # al = buf[left]
    movb (%rsi,%rdx,1), %bl       # bl = buf[right]
    movb %bl, (%rsi,%rcx,1)         # buf[left] = bl
    movb %al, (%rsi,%rdx,1)           # buf[right] = al
    inc %rcx
    dec %rdx
    jmp reverse_loop
reverse_done:
```

Each pass through the loop swaps the two outer characters and moves both
indices one step closer to the middle, exactly like reversing a physical
row of books by swapping the two ends inward — the loop naturally
terminates the instant the two indices meet (odd-length string) or cross
(even-length string), which `cmp %rcx, %rdx` / `jle` checks on every pass.

## What's next

You've now written every category of "ordinary program logic" this
section set out to cover: arithmetic, loops, functions, arrays, strings.
[Chapter 14](./14-reading-compiler-output.md) turns the lens around —
instead of writing assembly by hand, you'll compile real Rust/C code and
read the assembly *it* produces, using everything from this chapter to
recognize the exact same patterns.
