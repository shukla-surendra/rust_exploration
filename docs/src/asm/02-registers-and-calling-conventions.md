# 2. Registers & Calling Conventions: x86-64 vs aarch64

Every later chapter assumes this vocabulary. The single biggest gotcha
between the two architectures is **where the return address lives** —
get this wrong and naked functions/context switches break in ways that
are painful to debug.

## General-purpose registers, side by side

| Role | x86-64 | aarch64 |
|---|---|---|
| General purpose | `rax`, `rbx`, `rcx`, `rdx`, `rsi`, `rdi`, `r8`–`r15` (16 total, 64-bit) | `x0`–`x30` (31 total, 64-bit; `w0`–`w30` are their low 32 bits) |
| Stack pointer | `rsp` | `sp` (has alignment rules — must stay 16-byte aligned, enforced by hardware on some instructions) |
| Frame/base pointer | `rbp` (conventional, not hardware-special) | `x29` (conventional, called `fp`) |
| **Return address** | **pushed onto the stack by `call`, popped by `ret`** | **stored in a register (`x30`, called `lr` — "link register") by `bl`, read by `ret`** |
| Zero register | none (must `xor reg, reg`) | `xzr`/`wzr` — reads as zero, writes are discarded |
| Program counter | `rip` (not directly readable/writable by ordinary instructions) | `pc` (similarly restricted, but more directly involved in some addressing modes — see `adrp` below) |

**The return-address difference is the one to really internalize:**
x86's `call`/`ret` push/pop the stack automatically — a function can
freely call other functions without extra care, because each `call`
manages its own stack slot. aarch64's `bl` ("branch with link") instead
overwrites `x30` with the return address — call a *second* function
before you're done using the first return address, and you've just
clobbered it, unless you explicitly saved `x30` to the stack yourself
first. This is exactly why any aarch64 function that itself calls
another function needs a prologue that pushes `x30` (and `x29`) onto
the stack, and a matching epilogue that restores them — the compiler
does this for you in ordinary Rust functions, which is precisely what a
`naked` function (previous chapter) does *not* get for free.

## Calling conventions: how arguments and return values move

| | x86-64 (System V AMD64 ABI — Linux/macOS) | aarch64 (AAPCS64) |
|---|---|---|
| Integer/pointer args, in order | `rdi`, `rsi`, `rdx`, `rcx`, `r8`, `r9` | `x0`–`x7` |
| Return value | `rax` (`rax:rdx` for 128-bit) | `x0` (`x0:x1` for 128-bit) |
| Caller-saved (clobbered by any call) | `rax`, `rcx`, `rdx`, `rsi`, `rdi`, `r8`–`r11` | `x0`–`x18` |
| Callee-saved (must be preserved) | `rbx`, `rbp`, `r12`–`r15` | `x19`–`x28`, `x29`, `x30` |
| Stack alignment on `call`/`bl` | 16 bytes | 16 bytes |

This table is what `clobber_abi("C")` (previous chapter) encodes for
you automatically — the compiler already knows which registers a
"normal" function call is allowed to trash, per architecture, so your
`asm!` block doesn't have to hand-list them.

## `adrp`/`:lo12:` — the aarch64 address-loading idiom

This exact pattern is already in `hello-kernel`'s `_start`:

```rust
"adrp x0, __stack_top",
"add x0, x0, :lo12:__stack_top",
```

aarch64 instructions are fixed 32 bits wide, which isn't enough to
encode an arbitrary 64-bit address directly in one instruction. `adrp`
computes the address of the *4 KB page* containing `__stack_top`,
PC-relative; `add x0, x0, :lo12:__stack_top` then adds the low 12 bits
(the offset within that page) — together they reconstruct the full
address in two instructions. x86-64 doesn't need this two-step dance
for most addressing — `mov rax, __stack_top` (with the linker filling
in a 64-bit immediate or using RIP-relative addressing) typically
suffices in one instruction, though position-independent code has its
own RIP-relative conventions worth knowing exist if you go further than
this section covers.

## Why this chapter matters for every one that follows

- [Interrupts & Exceptions](./04-interrupts-and-exceptions.md)'s ISR
  stubs must save/restore *every* caller-saved register by hand (the
  CPU doesn't know which registers the interrupted code was using) —
  the caller-saved/callee-saved split above is exactly what determines
  which registers an ISR stub cannot skip saving.
- [Context Switching](./07-context-switching.md) needs the full
  register set (including `x30`/`lr` on aarch64, or the return address
  implicitly on the x86-64 stack) saved per task — this table is the
  literal list of what a `TaskContext` struct has to hold, on both
  architectures.
- [Syscall Entry & Exit](./06-syscall-entry-exit.md)'s argument-passing
  convention (which register holds the syscall number, which hold its
  arguments) is built directly on this chapter's calling-convention
  table.
