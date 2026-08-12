# 10. Functions & the Stack

## The stack, as a spring-loaded stack of plates

The stack is a region of memory a dedicated register always points at —
`rsp` on x86-64, `sp` on ARM64 — treated as growing **downward**, toward
lower addresses, every time something is pushed onto it. Picture a
spring-loaded plate dispenser: you can only add or remove from the top,
and the "top" pointer (the stack pointer register) moves automatically as
you do. That "only touch the top" discipline is exactly what makes it
cheap — no searching, no bookkeeping beyond one register.

x86-64 has dedicated instructions for this:

```asm
push %rax     # rsp -= 8; store rax at the new rsp
pop %rbx       # load from rsp into rbx; rsp += 8
```

ARM64 has no `push`/`pop` — you move `sp` and use `str`/`ldr` yourself, or
(far more common in practice) the paired `stp`/`ldp` (**st**ore/**l**oad
**p**air) which moves two registers at once, and is exactly what you'll
see in every ARM64 function prologue below:

```asm
stp x29, x30, [sp, #-16]!    // push x29 and x30 together, rsp -= 16 first
ldp x29, x30, [sp], #16       // pop them back, rsp += 16 after
```

The `!` and the position of `#-16` matter: `[sp, #-16]!` means "subtract
16 from `sp` *first*, then use that as the address" (a **pre-indexed**
store) — while `[sp], #16` on the load means "use `sp` as the address
*first*, then add 16 to `sp` afterward" (**post-indexed**). Together
they're the standard matched push/pop-a-pair idiom.

## `call`/`ret` vs. `bl`/`ret`: the difference that finally matters

This was previewed in
[Chapter 6](./06-registers-plain-english.md#the-single-biggest-structural-difference-where-the-return-address-lives)
— here's where it becomes concrete.

**x86-64's `call target`** does two things atomically: pushes the address
of the instruction right after the `call` onto the stack, then jumps to
`target`. **`ret`** does the reverse: pops that address off the stack and
jumps to it. Because the return address lives on the stack, calls nest
correctly automatically — a function can call another function, which
calls a third, and each `call` manages its own return-address slot with
zero extra work from you.

**ARM64's `bl target`** ("**b**ranch with **l**ink") jumps to `target` and
puts the return address in a single register, `x30` (the "link
register," `lr`) — not the stack. **`ret`** jumps to whatever address is
currently in `x30`. This works perfectly for a function that calls nobody
else (a **leaf function**) — but the moment that function itself needs to
`bl` somewhere else, the *new* return address overwrites `x30`, destroying
the old one, unless you explicitly saved it first. That's exactly what
`stp x29, x30, [sp, #-16]!` at the top of a non-leaf ARM64 function is for
— and it's the one extra step x86-64 code never needs, since the stack
already did that job automatically.

## A worked, recursive example: factorial

Recursion is the perfect way to see this land, because a recursive
function calls *itself* — the return address genuinely needs saving at
every level, or the whole chain breaks. This exact program was built and
run on Linux x86-64, Linux ARM64, and macOS ARM64, all three producing
`120` (5! = 5×4×3×2×1) via the exit code:

```asm
# x86-64
.global _start
.section .text
_start:
    mov $5, %rdi
    call factorial
    mov %rax, %rdi
    mov $60, %rax
    syscall

factorial:
    cmp $1, %rdi
    jle base_case
    push %rdi          # save n — it's still needed after the recursive call returns
    dec %rdi
    call factorial       # recurse: factorial(n-1), result comes back in rax
    pop %rdi
    imul %rdi, %rax        # rax = n * factorial(n-1)
    ret
base_case:
    mov $1, %rax
    ret
```

```asm
// ARM64 (Linux; swap the exit sequence per Chapter 5 for macOS)
.global _start
.section .text
_start:
    mov x0, #5
    bl factorial
    mov x8, #93
    svc #0

factorial:
    cmp x0, #1
    b.le base_case
    stp x0, x30, [sp, #-16]!   // save n AND the return address together
    sub x0, x0, #1
    bl factorial                // recurse: factorial(n-1), result comes back in x0
    ldp x1, x30, [sp], #16       // restore n into x1, and lr
    mul x0, x0, x1                // x0 = n * factorial(n-1)
    ret
base_case:
    mov x0, #1
    ret
```

## Why `push %rdi` alone is enough on x86-64, but ARM64 needs `stp x0, x30`

Look closely at what each version saves across the recursive call. The
x86-64 version only pushes `%rdi` (the value of `n`) — it never touches
the return address at all, because `call`/`ret` already handled that via
the stack, invisibly, as covered above. The ARM64 version pushes **two**
things together, `x0` (n) and `x30` (the return address) — because
`bl`'s upcoming recursive call is about to overwrite `x30`, and without
saving it first, `ret` at the end of this very function call would jump to
the *wrong* place (specifically, back into the recursive call it just
made, not back to *this* function's own caller). This single side-by-side
comparison is the entire "why does ARM64 assembly always seem to have this
`stp x29, x30` boilerplate at the top of every function" question,
answered concretely.

## Calling conventions: which registers hold what, by agreement

"Calling convention" (or **ABI** — Application Binary Interface) is simply
the agreed-upon rulebook for where arguments and return values go, so that
code compiled independently — by different compilers, even in different
languages — can still call each other correctly. You've been using it
implicitly (`%rdi`/`x0` for the first argument, `%rax`/`x0` for the return
value) throughout this chapter:

| | x86-64 (System V ABI — Linux/macOS) | ARM64 (AAPCS64) |
|---|---|---|
| Args 1–6, in order | `rdi`, `rsi`, `rdx`, `rcx`, `r8`, `r9` | `x0`–`x7` |
| Return value | `rax` | `x0` |
| Caller-saved (a call may trash these — save them yourself first if you still need them after) | `rax`, `rcx`, `rdx`, `rsi`, `rdi`, `r8`–`r11` | `x0`–`x18` |
| Callee-saved (a called function must restore these before returning) | `rbx`, `rbp`, `r12`–`r15` | `x19`–`x28`, `x29`, `x30` |
| Stack alignment required at a `call`/`bl` | 16 bytes | 16 bytes |

This table is exactly what makes [Chapter 11](./11-calling-c-from-asm.md)
possible at all — calling a real C library function like `printf` only
works because both your hand-written assembly and the compiler that built
libc agree on this same rulebook.

## What's next

Now that a function call convention exists, the natural next step is
calling a function you *didn't* write — a real C library function.
[Chapter 11](./11-calling-c-from-asm.md) does exactly that, and runs
straight into a genuine, documented difference between Linux's and
Apple's ARM64 calling conventions that isn't in the table above, caught
by testing it directly.
