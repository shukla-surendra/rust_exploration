# 8. `hello-kernel`: Build, Editions & the Linker Script

> Continues the [`hello-kernel` case study](./07-hello-kernel-overview.md)
> — imported from a separate project, not part of this repo.

## The target and the linker wiring

```toml
# .cargo/config.toml
[build]
target = "aarch64-unknown-none"

[target.aarch64-unknown-none]
rustflags = ["-C", "link-arg=-Tlinker.ld"]
runner = "qemu-system-aarch64 -M virt -cpu cortex-a72 -nographic -kernel"
```

`aarch64-unknown-none` is the target from chapter 7's requirements — "a
bare metal target with no OS, no libc, nothing but the raw CPU ABI."
`-C link-arg=-Tlinker.ld` passes `-T` (the "use this linker script"
flag) straight through to the linker, handing control of the entire
memory layout to `linker.ld` (below) instead of the default host
linker's assumptions about a normal OS process. `runner` is what lets
plain `cargo run` transparently become "build, then boot the result in
QEMU" — no separate manual `qemu-system-aarch64` invocation needed.

## `Cargo.toml`

```toml
[package]
name = "hello-kernel"
version = "0.1.0"
edition = "2024"

[[bin]]
name = "hello-kernel"
path = "src/main.rs"

[profile.dev]
panic = "abort"

[profile.release]
panic = "abort"
```

| Section | What it does |
|---|---|
| `[package]` | Metadata. `name` is the crate/binary name; `version` is a required semver string even for an unpublished crate; `edition` picks which edition's language rules apply (below). |
| `[[bin]]` | Explicitly declares the binary target. Cargo would infer this automatically from `src/main.rs` anyway (name/path already match Cargo's defaults) — this block only starts to matter once the binary name needs to differ from the package name, or there are multiple `[[bin]]` targets. |
| `[profile.dev]` / `[profile.release]` | Per-profile compiler settings — `dev` for plain `cargo build`/`cargo run`, `release` for `--release`. Both set `panic = "abort"` here. |

**Why `panic = "abort"`:** Rust's default panic strategy is `"unwind"`
— walking back up the call stack running destructors, which needs
runtime support (a personality function, landing pads) normally
provided by `std`/`libgcc`. This crate is `#![no_std]` with no OS
underneath to catch an unwind anyway, so both profiles set `"abort"`
instead: a panic just calls the panic handler (chapter 10) and halts —
no stack walking at all. Both profiles need it set explicitly here;
`panic = "abort"` under `[profile.release]` alone wouldn't let plain
`cargo run` (the `dev` profile) work without `--release`.

## Editions: `2021` vs `2024`

An **edition** is a per-crate, opt-in bundle of language changes — new
keywords, changed defaults, stricter syntax — that lets the language
evolve without breaking code written under an older edition. Edition is
**not** a compiler version: one up-to-date `rustc`/Cargo compiles 2015,
2018, 2021, and 2024-edition crates all in the same build, even mixed
in one dependency graph, because each crate carries its own `edition =
"..."` and the compiler applies only that crate's rules to it. Bumping
edition is something you opt into per crate (often via `cargo fix
--edition`), not something that happens automatically on a toolchain
update.

The 2021 → 2024 changes most relevant to a project like this:

- **Unsafe attributes are required.** Attributes with real safety
  implications — `no_mangle`, `export_name`, `link_section` — must be
  written as `#[unsafe(no_mangle)]` in the 2024 edition, instead of the
  old bare `#[no_mangle]`. **This project already uses that syntax** —
  `#[unsafe(no_mangle)]` and `#[unsafe(naked)]` on `_start`/`kernel_main`
  in `src/main.rs` (chapter 9). The syntax was stabilized independent of
  edition and compiles fine under `edition = "2021"` too (as it does
  here) — 2024 is what makes it *mandatory* rather than optional. If
  this crate bumped to `edition = "2024"`, nothing in `main.rs` would
  need to change, since it already writes 2024-style unsafe attributes.
- **`unsafe extern` blocks** — foreign function *declarations* inside an
  `extern "C" { ... }` block must be marked `unsafe extern "C" { ... }`
  in 2024. Not triggered here — this crate *defines* `extern "C"`
  functions (like `kernel_main`), rather than declaring external ones.
- **Tighter `if let`/`while let` temporary scoping** — temporaries
  created in the condition now drop at the end of that arm rather than
  living through the whole `if`/`else`, fixing a class of surprising
  lifetime-extension bugs (e.g. holding a lock guard longer than
  expected).
- `gen` becomes a reserved keyword, for an eventual generator/coroutine
  syntax; assorted smaller changes (`Future`/`IntoFuture` added to the
  prelude — see [Async Rust](../async-rust/00-is-it-in-the-language-or-not.md)
  for what that trait is — RPITIT capture-rule refinements, Cargo
  resolver defaults).

None of the 2024-only changes besides the unsafe-attribute syntax
(already adopted) affect this project — which is why `edition = "2021"`
was a perfectly reasonable choice for it even while writing 2024-style
syntax.

## `linker.ld` — the memory map, line by line

The linker script controls two things: where the ELF's entry point is
recorded, and where every byte of the binary ends up in memory. Since
QEMU loads this ELF with no bootloader in between (chapter 9), the
linker script is effectively defining the memory map of the entire
running machine.

```
ENTRY(_start)

SECTIONS
{
    . = 0x40080000;

    .text : {
        KEEP(*(.text._start))
        *(.text .text.*)
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

    .data : {
        *(.data .data.*)
    }

    .bss : {
        __bss_start = .;
        *(.bss .bss.*)
        *(COMMON)
        __bss_end = .;
    }

    . = ALIGN(16);
    . = . + 0x10000; /* 64 KiB stack */
    __stack_top = .;
}
```

| Line | What it does |
|---|---|
| `ENTRY(_start)` | Sets the ELF header's `e_entry` field to `_start`'s address. This is the one field QEMU reads with `-kernel` to know what address to set the CPU's program counter to. Nothing runs unless this points at the right place. |
| `. = 0x40080000;` | Sets the linker's *location counter* — the address where the next byte gets placed — to `0x40080000`. Everything from here is laid out starting at that address. It's a free choice; `0x40080000` is a conventional offset into the `virt` machine's RAM (which starts at `0x40000000`), leaving the low 512 KiB free. |
| `.text : { KEEP(*(.text._start)) *(.text .text.*) }` | The code section. `KEEP(*(.text._start))` forces `_start`'s code (which the compiler puts in its own `.text._start` subsection because of `#[unsafe(no_mangle)]` on a function named `_start`) to be placed **first**, and stops the linker's dead-code stripping from ever discarding it. Because it's first, right after `.` was set to `0x40080000`, `_start`'s first instruction lands at exactly that address — matching `ENTRY(_start)`. |
| `.rodata : { ... }` | Read-only data (string literals like `"Hello, kernel!\n"`), placed right after `.text`. |
| `.data : { ... }` | Initialized mutable statics, right after `.rodata` (none exist yet in this program). |
| `.bss : { __bss_start = .; ... __bss_end = .; }` | Reserves space for zero-initialized statics. `__bss_start`/`__bss_end` are linker-defined symbols marking its bounds — exported so future startup code could memset this range to zero before `kernel_main` runs (real kernels always do this; unused here since there's no `.bss` content yet). |
| `. = ALIGN(16); . = . + 0x10000;` | Advances the location counter by 64 KiB, 16-byte aligned. Emits no section contents — just reserves address space above the image for a stack to grow into. |
| `__stack_top = .;` | Defines `__stack_top` as the address right after that reserved region. aarch64 stacks grow *downward*, so this is the value `_start` loads into `sp` to initialize the stack (chapter 9). |

**Exactly where does execution begin?** The very first instruction the
CPU executes is `_start`'s first instruction, at `0x40080000`. Nothing
runs before it — no bootloader, no reset-vector code, no firmware
handoff. QEMU sets the guest CPU's program counter directly to the
ELF's `e_entry` value the moment emulation starts, because of `-kernel`.
The chain: `ENTRY(_start)` in `linker.ld` → recorded in the ELF header →
read by QEMU at startup → written into the CPU's PC → `_start`'s first
instruction executes. Chapter 9 picks up exactly there.
