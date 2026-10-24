# 14. Reading Compiler-Generated Assembly

Every chapter so far wrote assembly by hand. This one goes the other
direction: take ordinary C (the same technique applies to Rust — see the
note at the end), ask the compiler to stop before the final machine-code
step, and read exactly what it produced. This is, in practice, the single
most useful skill this whole section builds toward — it's how you debug a
crash, understand what `-O2` actually bought you, or resolve an ABI
disagreement exactly like [Chapter 11](./11-calling-c-from-asm.md)'s
Apple-variadic-arguments discovery.

## The `-S` flag: stop at assembly, don't finish the build

```bash
gcc -S -O0 add.c -o add_O0.s        # Linux / gcc
clang -S -O0 add.c -o add_O0.s       # macOS / clang
```

`-S` tells the compiler "produce a `.s` file and stop" — the exact same
kind of file you've been hand-writing all section, just compiler-authored.

## `-O0` vs `-O2`: watching the optimizer actually think

Take the simplest possible function:

```c
int add(int a, int b) {
    return a + b;
}
```

**Unoptimized (`-O0`), Linux x86-64 — GCC deliberately does the dumbest
correct thing, to keep debugging predictable:**

```asm
add:
    endbr64
    pushq   %rbp
    movq    %rsp, %rbp
    movl    %edi, -4(%rbp)    # spill argument a to the stack
    movl    %esi, -8(%rbp)     # spill argument b to the stack
    movl    -4(%rbp), %edx       # reload a
    movl    -8(%rbp), %eax        # reload b
    addl    %edx, %eax
    popq    %rbp
    ret
```

**Optimized (`-O2`), same function, same platform:**

```asm
add:
    endbr64
    leal    (%rdi,%rsi), %eax
    ret
```

Two arguments in, one add, one return — the entire unoptimized version's
stack-spilling ceremony is simply gone, because `-O2`'s optimizer proved
it was never necessary in the first place (nothing else needed `a` or `b`
to live anywhere but a register for their whole, brief lifetime).

Note the trick in the `-O2` version: `leal (%rdi,%rsi), %eax` uses
**`lea`** — introduced back in [Chapter 3](./03-hello-world-linux-x86-64.md)
purely for computing addresses — to do plain **arithmetic** instead.
`lea`'s whole job is computing `base + index` without touching memory
([Chapter 8](./08-memory-and-addressing.md)), and `rdi + rsi` is
arithmetically identical whether you're computing an address or just
adding two numbers — so the compiler reuses it as a free add instruction.
This exact trick is a very common thing to spot once you know to look for
it, and it's a good example of how compiler-generated assembly often does
something a human wouldn't think to write by hand, purely because the
optimizer is searching a much larger space of equivalent instruction
sequences than a person would bother to.

**The same function, `-O0`, on macOS ARM64:**

```asm
_add:
    sub  sp, sp, #16
    str  w0, [sp, #12]     # spill argument a
    str  w1, [sp, #8]        # spill argument b
    ldr  w8, [sp, #12]         # reload a
    ldr  w9, [sp, #8]            # reload b
    add  w0, w8, w9
    add  sp, sp, #16
    ret
```

**`-O2`:**

```asm
_add:
    add  w0, w1, w0
    ret
```

Same story, same shape of improvement, different instruction vocabulary —
exactly the pattern the whole rest of this section has built up.

**One label you'll see and can safely ignore for now**: `endbr64`, at the
very top of every x86-64 function GCC/Clang emits by default. It's part
of **CET (Control-flow Enforcement Technology)**, a hardware security
feature marking valid jump/call targets to make certain exploit techniques
harder — it does nothing to the function's actual logic, and none of the
hand-written examples in this section bothered to include it, since it's
purely a hardening measure, not something correctness depends on.

## Disassembling an already-built binary: `objdump` and `otool`

Sometimes you don't have the source at all — just a compiled binary (your
own, or someone else's) — and want to see what's actually inside it.

```bash
objdump -d add.o     # Linux — works on ELF objects and executables
```

```
0000000000000000 <add>:
   0:   f3 0f 1e fa             endbr64
   4:   8d 04 37                lea    (%rdi,%rsi,1),%eax
   7:   c3                      ret
```

```bash
otool -tv hello_macos_arm64     # macOS — works on Mach-O objects and executables
```

```
_main:
0000000100000390    mov    x0, #0x1
0000000100000394    adrp   x1, 4 ; 0x100004000
0000000100000398    add    x1, x1, #0x0
000000010000039c    mov    x2, #0xe
...
```

That second output is worth pausing on: it's `otool` disassembling
[Chapter 5](./05-hello-world-macos.md)'s own hand-written hello-world
binary, and every single instruction in it matches what was hand-typed —
proof that "assembly → machine code" really is the mechanical,
1-to-1, nothing-hidden translation [Chapter 1](./01-what-is-a-cpu.md)
claimed it was. Also notice the addresses on the left (`0000000100000390`
and up) — this is the actual memory address each instruction will sit at
once loaded, and it's exactly this kind of address a debugger
([Chapter 15](./15-debugging.md)) shows you when a program crashes.

`-d` (objdump) and `-t` (otool) mean "just disassemble"; the extra `v`
on `otool -tv` additionally resolves symbol names (`_main`) instead of
showing raw addresses everywhere.

## The Rust angle, briefly

Exactly the same technique, different flag: `rustc --emit asm -O
file.rs` (or, much more pleasantly, the online tool **Compiler
Explorer**, godbolt.org, which does this live for Rust, C, and dozens of
other languages side-by-side, with the generated assembly interactively
linked back to the source line that produced it — genuinely worth
bookmarking once this technique clicks). Everything you've learned in
this section — register names, the two calling conventions, `mov`/`ldr`
vs. `lea`/`adr`, `-O0`'s stack-spilling vs. `-O2`'s directness — reads
identically whether the source was C or Rust; the compiler backend
(LLVM, for both `clang` and `rustc`) is doing fundamentally the same job
either way.

## What's next

You can now read what a compiler produces. [Chapter 15](./15-debugging.md)
covers the other essential real-world skill — stepping through *running*
code, live, one instruction at a time, watching registers and memory
change as it goes.
