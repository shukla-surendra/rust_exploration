# 7. Context Switching: the Assembly That Makes a Scheduler Possible

A scheduler's actual job, mechanically, is: save every piece of CPU
state the currently-running task needs to resume later, restore another
task's previously-saved state into the CPU, and jump into it. Every
scheduler concept (round-robin, priorities, preemption) sits on top of
this one operation — see
[OxideOS Study Journal 05](../oxideos/study/05_processes.md) for the
`Task`/scheduler structure this section's assembly ultimately serves.

## What has to be saved — literally [Chapter 2](./02-registers-and-calling-conventions.md)'s table

A task's full saved context is exactly: every general-purpose register,
the stack pointer, and (per
[Privilege Levels](./03-privilege-levels-and-mode-transitions.md)) the
saved program-counter/status-register state needed to resume execution
at the right privilege level — plus, per
[Paging & MMU Setup](./05-paging-and-mmu-setup.md), which address space
the task belongs to.

```rust
// x86-64 shape
#[repr(C)]
struct TaskContext {
    rbx: u64, rbp: u64, r12: u64, r13: u64, r14: u64, r15: u64,  // callee-saved only — see below
    rsp: u64,
    cr3: u64,   // this task's page table root (Chapter 5)
}
```

**Only callee-saved registers, not all of them.** This is the detail
that trips people up: a context switch is implemented as an ordinary
function call (`switch_to(next_task)`) — by the calling convention
itself ([Chapter 2](./02-registers-and-calling-conventions.md)), the
*caller* is already responsible for saving any caller-saved register it
still needs across the call. The context-switch function only has to
explicitly save the registers a normal `call` wouldn't already protect
— the callee-saved set — which is a genuine, deliberate optimization:
letting the ordinary calling convention do half the work for free.
(An *interrupt*-driven preemption, by contrast, can't rely on this — the
interrupted code didn't choose to call anything, so
[Chapter 4](./04-interrupts-and-exceptions.md)'s ISR stub must save
*everything*, caller-saved registers included.)

## x86-64: the switch function

```
switch_to:
    ; save the OLD task's context (rdi = &mut old_context, by calling convention)
    mov [rdi + 0x00], rbx
    mov [rdi + 0x08], rbp
    mov [rdi + 0x10], r12
    mov [rdi + 0x18], r13
    mov [rdi + 0x20], r14
    mov [rdi + 0x28], r15
    mov [rdi + 0x30], rsp

    ; load the NEW task's context (rsi = &new_context)
    mov cr3, [rsi + 0x38]          ; switch address space FIRST
    mov rsp, [rsi + 0x30]
    mov r15, [rsi + 0x28]
    mov r14, [rsi + 0x20]
    mov r13, [rsi + 0x18]
    mov r12, [rsi + 0x10]
    mov rbp, [rsi + 0x08]
    mov rbx, [rsi + 0x00]
    ret                             ; pops the NEW task's saved return address off its own stack!
```

The last `ret` is the trick that makes this work with no special
instruction: because `rsp` was just switched to the new task's stack,
the address `ret` pops is whatever that task's stack has sitting at its
top — for a task that's never run before, that's deliberately
pre-populated (at task creation) to point at wherever it should first
execute; for a task that's merely resuming, it's the return address
*from the last time this exact function called `switch_to` on its
behalf*. A context switch, from the CPU's perspective, is just an
ordinary function call that happens to return into a different task
than the one that made it.

## aarch64: the same trick, with `x30` explicit

```
switch_to:
    ; x0 = &mut old_context, x1 = &new_context
    stp x19, x20, [x0, #0x00]
    stp x21, x22, [x0, #0x10]
    ; ... x23-x28 similarly ...
    stp x29, x30, [x0, #0x50]       ; frame pointer AND link register
    mov x2, sp
    str x2, [x0, #0x60]

    ldr x2, [x1, #0x68]             ; new task's TTBR0_EL1
    msr ttbr0_el1, x2
    isb

    ldr x2, [x1, #0x60]
    mov sp, x2
    ldp x29, x30, [x1, #0x50]
    ; ... restore x23-x28, x21-x22, x19-x20 ...
    ret                              ; branches to x30 — the NEW task's saved link register
```

Structurally identical to the x86-64 version, with two aarch64-specific
details: `x30` (the link register — [Chapter 2](./02-registers-and-calling-conventions.md))
must be explicitly saved/restored since it's an ordinary register here,
not something the stack manages implicitly; and only `TTBR0_EL1` needs
switching (per [Chapter 5](./05-paging-and-mmu-setup.md)'s split address
space) — `TTBR1_EL1`, the kernel half, stays fixed across every task.

## The invariant underneath both

Every context switch, on either architecture, is fundamentally: *save
just enough state that "calling a function and having it later return"
is indistinguishable from "this task's execution is paused and later
resumed."* The scheduler's entire illusion of multiple simultaneously-
running tasks rests on this one function, called from a timer interrupt
(preemptive scheduling) or a voluntary yield — both of which just need
to arrange for `switch_to` to be called at all, the switch itself is
identical either way.
