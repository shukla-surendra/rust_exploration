# 1. Inline Assembly in Rust: the `asm!` Family

Three related macros, three different jobs — all from `core::arch`
(available even in `#![no_std]`, no crate needed):

| Macro | Use for |
|---|---|
| `asm!` | A few instructions inline inside an ordinary Rust function |
| `global_asm!` | A standalone assembly routine at module/file scope, outside any function |
| `#[unsafe(naked)]` + `naked_asm!` | An entire function body that's pure assembly, with **no Rust-generated prologue/epilogue** |

## `asm!` — instructions inline in a function

```rust
use core::arch::asm;

pub fn read_flags() -> u32 {
    let flags: u32;
    unsafe {
        asm!("mrs {0}, nzcv", out(reg) flags);   // aarch64: read condition flags
    }
    flags
}
```

Always `unsafe` — the compiler cannot verify what raw instructions do,
the same trust boundary as a raw pointer dereference (see
[Ownership, Borrowing & Lifetimes](../workbook/02-ownership-borrowing-lifetimes.md)
for why `unsafe` exists at all: it doesn't turn off the rules, it moves
the proof obligation from the compiler to you).

### Operands: how Rust values get into/out of your assembly

```rust
let a: u64 = 5;
let mut result: u64;
unsafe {
    asm!(
        "add {out}, {a}, 1",
        a = in(reg) a,
        out = out(reg) result,
    );
}
// result == 6
```

- `in(reg) a` — put `a`'s value into some register the compiler picks;
  the template refers to it as `{a}`.
- `out(reg) result` — after the asm runs, read whatever register `{out}`
  became and store it in `result`. The compiler assumes the old value in
  that register is now garbage.
- `inout(reg) x` — a register that's both read as input and written as
  output, same register both times (for a value updated in place).
- `lateout(reg)` — like `out`, but tells the compiler this register is
  only written *after* all inputs have been consumed — lets the
  compiler safely reuse an input register for this output, sometimes
  avoiding wasted register pressure.

Letting the compiler pick the register (`reg`) is almost always right —
it can then allocate registers optimally around your asm block. Force a
*specific* register only when the instruction itself requires it:

```rust
unsafe {
    asm!("out dx, al", in("dx") port, in("al") value);   // x86: `out` hardcodes which regs
}
```

### `options()` — telling the compiler what your asm doesn't touch

```rust
unsafe {
    asm!("nop", options(nomem, nostack, preserves_flags));
}
```

- `nomem` — this asm doesn't read/write memory; lets the compiler keep
  values cached in registers around the block instead of conservatively
  reloading everything from memory before and after.
- `nostack` — doesn't push/pop the stack.
- `preserves_flags` — doesn't clobber CPU condition flags, so the
  compiler doesn't need to save/restore them around the block.
- `noreturn` — this asm block never falls through (jumps/halts instead)
  — lets the compiler treat code after it as unreachable, same category
  as a function returning `!` (see
  [Error Handling](../foundation/error-handling.md) for `!`, the "never"
  type, used the same way for `process::exit`/`loop {}`).

These are performance hints with real teeth: get one wrong (claim
`nomem` while actually writing memory) and the compiler is free to
generate code that miscompiles — the same "you're promising something
the compiler can't verify" contract as any other `unsafe` code.

## `global_asm!` — a whole routine, outside any function

```rust
core::arch::global_asm!(
    ".global my_routine",
    "my_routine:",
    "ret",
);
```

Used for things that need to exist as standalone symbols the linker can
reference — an entire exception handler table, or (as in the next
section) a naked entry point in projects that don't use `#[unsafe(naked)]`.

## Naked functions — a whole function body with zero Rust-generated code

This is the one you've already seen working code for —
`hello-kernel`'s `_start`:

```rust
#[unsafe(no_mangle)]
#[unsafe(naked)]
pub unsafe extern "C" fn _start() -> ! {
    naked_asm!(
        "adrp x0, __stack_top",
        "add x0, x0, :lo12:__stack_top",
        "mov sp, x0",
        "bl kernel_main",
        "1:",
        "wfe",
        "b 1b",
    )
}
```

Full walkthrough of exactly this function in
[Systems, Chapter 9](../systems/09-hello-kernel-boot-to-execution.md).
The reason it must be `naked`: an ordinary Rust function's compiler-
generated prologue touches the stack (to save the frame pointer, spill
registers) — but at the very first instruction the CPU ever executes,
`sp` is undefined. A normal function would corrupt memory at whatever
garbage address `sp` happens to hold; `naked_asm!` guarantees **the
function body is exactly what you wrote, nothing added before or
after** — the first job of that body has to be establishing a valid
stack, by hand, before anything else can safely run.

`#[unsafe(naked)]`/`#[unsafe(no_mangle)]` (2024-edition syntax) — see
[Systems, Chapter 8](../systems/08-hello-kernel-build-and-linking.md)
for the full 2021-vs-2024 edition explanation of why these need the
`unsafe(...)` wrapper now.

## `clobber_abi` — the easy way to say "this calls into normal Rust/C code"

```rust
unsafe {
    asm!("bl kernel_main", clobber_abi("C"));
}
```

Rather than manually listing every register a called function might
clobber, `clobber_abi("C")` tells the compiler "assume this asm follows
the C calling convention and clobbers everything that convention allows
a callee to clobber" — the compiler fills in the correct register list
for whichever target you're compiling for (different for x86-64 vs
aarch64 — see the next chapter), which is exactly the situation
`_start`'s `bl kernel_main` is in.
