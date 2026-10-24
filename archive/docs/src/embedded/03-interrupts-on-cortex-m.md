# 3. Interrupts on Cortex-M: the NVIC

[Assembly for OS Development, Chapter 4](../asm/04-interrupts-and-exceptions.md)
covered x86-64's IDT and aarch64's 16-slot exception vector table, both
requiring you to hand-write register-saving ISR stubs. Cortex-M's
**NVIC** (Nested Vectored Interrupt Controller) is simpler on exactly
the point that made those two painful.

## The vector table — a plain array of function pointers

```rust
#[link_section = ".vector_table.interrupts"]
#[used]
static INTERRUPTS: [Vector; 2] = [
    Vector { handler: TIM2_handler },
    Vector { handler: USART1_handler },
];
```

In practice `cortex-m-rt`'s `#[interrupt]` attribute generates this for
you:

```rust
#[interrupt]
fn TIM2() {
    // runs when TIM2's interrupt fires
}
```

Structurally the same idea as aarch64's vector table
([Assembly, Chapter 4](../asm/04-interrupts-and-exceptions.md)) — a
fixed table the hardware indexes into directly — but Cortex-M's table
has one entry **per specific peripheral interrupt source** (like x86's
IDT, one slot per cause) rather than aarch64's four broad categories
that a handler then has to disambiguate via `ESR_EL1`.

## The genuinely simpler part: hardware saves your registers for you

This is the real difference from every architecture in the assembly
section. On entering *any* exception, Cortex-M hardware automatically
pushes `r0`–`r3`, `r12`, the link register, the return address, and the
status register onto the stack **before your handler runs a single
instruction** — no hand-written prologue required, no `stp`/`push`
sequence to get right. Your interrupt handler can be an ordinary Rust
function; there's no `naked_asm!` involved at all, unlike
[hello-kernel's `_start`](../asm/01-inline-asm-in-rust.md) or
[Assembly, Chapter 4](../asm/04-interrupts-and-exceptions.md)'s hand-
rolled ISR stubs. `cortex-m-rt`'s `#[interrupt]` functions are
completely ordinary, safe(-ish) Rust — the hardware's own exception
entry mechanism does the job a naked function's assembly does on
x86-64/aarch64.

## Priorities, without a full external PIC/GIC

Cortex-M builds priority levels directly into the NVIC — no separate
chip like x86's 8259A PIC
([hardware reference](../oxideos/study/hardware/8259A_pic.md)) required:

```rust
use cortex_m::peripheral::NVIC;

unsafe {
    NVIC::unmask(Interrupt::TIM2);
}
cortex_m::peripheral::NVIC::pend(Interrupt::TIM2);   // manually trigger, for testing
```

`unmask` enables a specific interrupt source (the NVIC-level analog of
[Assembly, Chapter 4](../asm/04-interrupts-and-exceptions.md)'s
`cli`/`sti`, but per-source instead of globally all-or-nothing).
Priority levels let a higher-priority interrupt preempt a lower-
priority handler already running — "nested," the N in NVIC — which is
exactly the primitive [RTIC](./07-rtic-alternative.md) builds an entire
concurrency model on top of.

## Critical sections — still the same underlying concern

```rust
cortex_m::interrupt::free(|_cs| {
    // interrupts disabled for the duration of this closure
});
```

Same fundamental need as
[Assembly, Chapter 4](../asm/04-interrupts-and-exceptions.md#enabling-and-disabling-interrupts)'s
`cli`/`daifset` — protecting shared state from being mutated by both
normal code and an interrupt handler simultaneously, the same "many
readers XOR one writer" concern from
[Ownership, Borrowing & Lifetimes](../workbook/02-ownership-borrowing-lifetimes.md).
`cortex_m::interrupt::free` wraps disable/re-enable in a closure so you
can't forget the matching re-enable — the same "can't forget to release
it" guarantee `MutexGuard`'s `Drop` gives you in
[Memory & Smart Pointers](../workbook/07-memory-and-smart-pointers.md),
applied to interrupt masking instead of a lock.
