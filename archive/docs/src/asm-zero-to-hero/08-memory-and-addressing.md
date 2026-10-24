# 8. Memory & Addressing Modes

## RAM, as a single very long street of numbered mailboxes

Forget "files" and "variables" for a moment — to the CPU, RAM is just a
huge sequence of numbered byte-sized boxes, from address `0` up to
whatever the machine has installed. Every single thing a running program
has — its instructions, a string literal, an array, a local variable — is
just sitting at *some* address on that street. A **pointer** (a term
you've already been using since Chapter 3's `lea msg(%rip), %rsi`) is
nothing more than a register holding one of those addresses.

An **addressing mode** is a way of telling an instruction "here's how to
compute the address I actually want," rather than always spelling out a
raw number. This chapter is the reference for every form you'll
encounter, on both architectures.

## The four addressing modes you'll actually use

**1. Immediate** — not really an address at all, a literal value baked
into the instruction. Already used constantly:

```asm
mov $5, %rax        # x86-64
mov x0, #5            // ARM64
```

**2. Register direct** — the value already lives in a register, no memory
touched:

```asm
mov %rax, %rbx      # x86-64
mov x1, x0             // ARM64
```

**3. Register indirect** — "the address to use is *whatever value sits
in* this register" — this is dereferencing a pointer:

```asm
mov (%rbx), %rcx    # x86-64: rcx = the value at the address in rbx
ldr x2, [x1]           // ARM64: x2 = the value at the address in x1
```

**4. PC-relative** — "the address is computed relative to where this very
instruction sits in memory" — used for referring to your own program's
data (like `msg`) without hard-coding an absolute address that would break
the moment the OS loads your program somewhere else:

```asm
lea msg(%rip), %rsi     # x86-64
adr x1, msg                // ARM64 (nearby data only, ±1MB or so)
adrp x1, msg@PAGE          // ARM64 (Mach-O syntax, arbitrary distance)
add  x1, x1, msg@PAGEOFF
adrp x1, msg                // ARM64 (ELF syntax, arbitrary distance)
add  x1, x1, :lo12:msg
```

## Why ARM64 needs two instructions for a "far" address, and x86-64 usually doesn't

This was flagged in [Chapter 4](./04-hello-world-linux-arm64.md) and
[Chapter 5](./05-hello-world-macos.md) — here's the actual reason, now
that [Chapter 1](./01-what-is-a-cpu.md)'s RISC/CISC distinction is in
place. Every ARM64 instruction is exactly 4 bytes (32 bits), no
exceptions. A 64-bit address obviously can't fit inside a 32-bit
instruction alongside the bits needed to say *which* instruction this even
is — there simply isn't room. ARM64's answer is `adrp` (address of page,
PC-relative): it computes the address of the **4KB page** containing your
target, which needs fewer bits since it ignores the address's low 12
bits — then a second instruction (`add ...:lo12:...` or `add
...@PAGEOFF`) adds those low 12 bits back in, reconstructing the exact
address across two 4-byte instructions.

x86-64 sidesteps this because its instructions are **variable length** (1
to 15 bytes) — an instruction that needs to embed a 64-bit immediate
address can simply *be longer*. RIP-relative addressing (`lea
msg(%rip), %rsi`) usually only needs a 32-bit *offset* from the current
instruction anyway (fine for anything within about 2GB of itself, which
covers virtually all real programs), so in practice it fits in one
instruction too — but for the rare case of a truly arbitrary 64-bit
address, x86-64 has that headroom in a way a fixed 4-byte ARM64
instruction structurally cannot.

## Indexed addressing: how arrays actually work under the hood

This is the pattern behind every array access you've ever written in a
higher-level language — `arr[i]` is, underneath, exactly this arithmetic:
**base address + (index × element size)**. Both architectures can do that
whole computation in a single load instruction. Verified working (built
and run on both Linux x86-64 and Linux ARM64, summing `{10, 20, 30, 40,
5}` and confirming the result `105` via exit code):

```asm
# x86-64: rsi = base address, rcx = index, element size 8 bytes
mov (%rsi,%rcx,8), %rax    # rax = *(rsi + rcx*8)
```

```asm
// ARM64: x1 = base address, x2 = index, element size 8 bytes (shift by 3 = *8)
ldr x4, [x1, x2, lsl #3]    // x4 = *(x1 + (x2 << 3))
```

`(%rsi,%rcx,8)` in AT&T syntax reads as "base register, index register,
scale factor" — the scale must be 1, 2, 4, or 8 (matching how many bytes
one element of that size takes: a byte, a 16-bit value, a 32-bit value, or
a 64-bit value). ARM64 has no dedicated "scale" field in its addressing
syntax — instead `lsl #3` (**l**ogical **s**hift **l**eft by 3 bits)
explicitly multiplies the index by 8 as part of the same instruction;
shifting left by 3 bits is the bit-level equivalent of multiplying by 2³ =
8, chosen here because our elements are 8-byte `.quad` values (see
[Chapter 13](./13-arrays-strings-algorithms.md) for the full worked
version of this array-sum program, including the loop around it).

## Stack vs. heap, in this same "numbered street" picture

Two more addresses worth naming here, ahead of
[Chapter 10](./10-functions-and-the-stack.md) needing them: the **stack**
(a region of memory a register — `rsp` on x86-64, `sp` on ARM64 — always
points at the current top of, growing *downward* toward lower addresses as
more gets pushed onto it) and the **heap** (memory your program requests
from the OS explicitly, at runtime, via a syscall like `mmap` or a
libc function like `malloc` built on top of one — nothing this section
does needs the heap directly, but it's the same "just an address on the
same street" concept once you get there in a higher-level language).

## What's next

Every ingredient for a real loop is now in place: values in registers
([Chapter 6](./06-registers-plain-english.md)), arithmetic
([Chapter 7](./07-instructions-101.md)), and addresses (this chapter).
[Chapter 9](./09-control-flow.md) is where `cmp` and a conditional jump
turn straight-line code into an actual loop.
