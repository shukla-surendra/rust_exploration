# 6. Syscall Entry & Exit: `int`/`syscall` vs `svc`

[Privilege Levels & Mode Transitions](./03-privilege-levels-and-mode-transitions.md)
covered the general Ring/EL crossing mechanism. This chapter is
specifically the userspace → kernel crossing a syscall performs, and
back — the exact instructions OxideOS's 80+ Linux-ABI-compatible
syscalls (see its [README](../oxideos/00-overview.md)) go through on
every single call.

## x86-64: two mechanisms, old and fast

**Legacy: `int 0x80`** — a software interrupt, routed through the same
IDT machinery as [Chapter 4](./04-interrupts-and-exceptions.md)'s
hardware interrupts. Userspace loads the syscall number into `rax` and
arguments into `rdi`/`rsi`/`rdx`/`r10`/`r8`/`r9` (note: `r10` takes the
4th argument's usual slot, *not* `rcx` — `syscall`, below, clobbers
`rcx`, so the convention avoids that register for syscall args even on
the `int 0x80` path, for consistency), then executes `int 0x80`. The CPU
does a full IDT lookup, privilege check, and `iretq`-style frame push —
correct, but relatively slow, since it's general-purpose interrupt
machinery being repurposed for a very hot path.

**Fast path: `syscall`/`sysret`** — dedicated instructions, no IDT
lookup at all:

```
; userspace side:
mov rax, syscall_number
mov rdi, arg1
mov rsi, arg2
; ... rdx, r10, r8, r9 for further args ...
syscall
; rax now holds the return value
```

```
; one-time kernel setup, at boot, via Model-Specific Registers:
mov rcx, 0xC0000082    ; IA32_LSTAR — where `syscall` jumps to
wrmsr
mov rcx, 0xC0000081    ; IA32_STAR — kernel/user CS:SS selectors to load
wrmsr
mov rcx, 0xC0000084    ; IA32_FMASK — RFLAGS bits to clear on entry
wrmsr
```

`syscall` jumps straight to the address in `LSTAR`, switches to Ring 0,
and stashes the return address in `rcx` (**not** the stack — worth
noting since it's the same "return address lives in a register, not
memory" pattern [Chapter 2](./02-registers-and-calling-conventions.md)
flagged for aarch64's `bl`/`x30`, here showing up as an x86-specific
exception to x86's usual push-to-stack convention). The kernel-side
handler must therefore save `rcx` immediately, same reasoning as any
ISR stub. `sysret` is the matching fast return.

## aarch64: one instruction, `svc`

```
; userspace side:
mov x8, syscall_number
mov x0, arg1
mov x1, arg2
; ... x2-x5 for further args ...
svc #0
; x0 now holds the return value
```

`svc` (SuperVisor Call) is a **synchronous exception** — it lands in
[Chapter 4](./04-interrupts-and-exceptions.md)'s exception vector table,
specifically the "synchronous, from a lower EL" slot. There's no
separate fast-path instruction the way x86-64 has `syscall` alongside
`int 0x80` — `svc` *is* the one and only mechanism, and it was designed
from the start to be cheap, so aarch64 never needed a second, faster
alternative the way x86 did.

The kernel-side handler reads `ESR_EL1` (Exception Syndrome Register) to
determine *why* it was entered (a `svc` vs. a page fault vs. an
undefined instruction all land in the same "synchronous exception"
vector slot) before dispatching to syscall-specific handling — a step
x86-64 doesn't need, since `int 0x80`/`syscall` are unambiguously
syscall-only entry points by construction.

## Side by side

| | x86-64 (`syscall`) | aarch64 (`svc`) |
|---|---|---|
| Syscall number register | `rax` | `x8` |
| Argument registers | `rdi`, `rsi`, `rdx`, `r10`, `r8`, `r9` | `x0`–`x5` |
| Return value register | `rax` | `x0` |
| Return address saved in | `rcx` (register, not stack) | `ELR_EL1` (system register) |
| Kernel entry point configured via | `IA32_LSTAR` MSR | exception vector table slot |
| Distinguishing "why was I entered" | implicit (dedicated instruction) | `ESR_EL1` (shared vector slot) |
| Return instruction | `sysret` (fast) or `iretq` (via `int 0x80`) | `eret` |

See [OxideOS Concepts, Chapter 4](../oxideos/oxide_cocepts/04_syscalls_and_usermode.md)
and [Study Journal 06](../oxideos/study/06_syscalls.md) for the full
x86-64 dispatch path, `int 0x80` through to `handle_syscall`, in real
working code.
