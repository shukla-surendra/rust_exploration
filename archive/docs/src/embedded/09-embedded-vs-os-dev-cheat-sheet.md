# 9. Embedded vs. OS Dev: Cheat Sheet

The whole comparison this section has been drawing chapter by chapter,
one table — connecting every section in this book that touches
low-level Rust.

| | Embedded (Cortex-M, this section) | OS Kernel Dev ([`hello-kernel`](../systems/07-hello-kernel-overview.md), [OxideOS](../oxideos/00-overview.md)) |
|---|---|---|
| Target | A microcontroller — the whole system | An application processor — hosts other programs |
| MMU / virtual memory | Usually none (optional MPU for protection only) | Yes — [Systems, Chapter 3](../systems/03-ram-and-virtual-memory.md), [Assembly, Chapter 5](../asm/05-paging-and-mmu-setup.md) |
| RAM budget | KB to low MB | MB to GB |
| Privilege levels | Usually one (or a simple 2-level Thread/Handler mode) | Rings/Exception Levels — [Assembly, Chapter 3](../asm/03-privilege-levels-and-mode-transitions.md) |
| Startup code | `cortex-m-rt` (generated) | Hand-written `_start` + linker script ([Assembly, Chapter 1](../asm/01-inline-asm-in-rust.md)) or a full bootloader chain |
| Interrupt entry | Hardware auto-saves registers — [Chapter 3](./03-interrupts-on-cortex-m.md) | Hand-written ISR stub, save every register — [Assembly, Chapter 4](../asm/04-interrupts-and-exceptions.md) |
| Peripheral access | PAC/HAL, typed — [Chapter 2](./02-pac-hal-and-registers.md) | Hand-written `read_volatile`/`write_volatile`, or an OS's own driver layer |
| Heap | Often none, or a small fixed-size one — [Chapter 5](./05-memory-constraints-and-heapless.md) | A real allocator ([OxideOS's bump/paging allocator](../oxideos/oxide_cocepts/03_memory_management.md)) |
| Concurrency | Embassy (async) or RTIC (priority-preemptive) — [Chapters 6](./06-async-embedded-with-embassy.md)–[7](./07-rtic-alternative.md) | A scheduler + context switching — [Assembly, Chapter 7](../asm/07-context-switching.md) |
| Runs other programs? | No — it *is* the program | Yes — userspace processes ([Assembly, Chapter 3](../asm/03-privilege-levels-and-mode-transitions.md), [OxideOS](../oxideos/00-overview.md)'s 80+ syscalls) |
| Typical dev workflow | `probe-rs` flash to real hardware, or QEMU | QEMU almost exclusively (`hello-kernel`), or QEMU + real-hardware install (OxideOS) |
| Failure mode of a bug | Often silent corruption (no MMU guard pages) | Often a fault/panic the kernel can catch and report |

## What genuinely carries over between the two

- **`#![no_std]`/`#![no_main]`, `unsafe`, volatile MMIO** — identical
  concepts, same reasons, in both worlds. Everything in
  [Systems, Chapter 1](../systems/01-prerequisites-bits-bytes-and-addressing.md)
  and [Chapter 10 of the `hello-kernel` case study](../systems/10-hello-kernel-uart-and-panics.md)
  transfers directly.
- **The `Future`/executor split** — [Async Rust, Chapter 0](../async-rust/00-is-it-in-the-language-or-not.md)'s
  core claim (language-level `async`/`.await` vs. external executor) is
  *proven* by the fact that the exact same language feature runs on
  `tokio` (a multi-threaded server) and Embassy (a single-core
  microcontroller with no OS at all) — about as different as two
  execution environments can be, unified by one trait.
- **Ownership/borrowing as a safety mechanism for hardware, not just
  memory** — [Chapter 2](./02-pac-hal-and-registers.md)'s
  `Peripherals::take()` and [Chapter 7](./07-rtic-alternative.md)'s
  compile-time-generated critical sections are both
  [Ownership, Borrowing & Lifetimes](../workbook/02-ownership-borrowing-lifetimes.md)'s
  rules, applied to *physical registers and interrupt priorities*
  instead of just RAM — the same discipline
  [Assembly, Chapter 4](../asm/04-interrupts-and-exceptions.md) enforces
  by hand (`cli`/`daifset` around a critical section) shows up here
  enforced by the type system instead.

## Which one to actually reach for

- **Building or extending an OS, hypervisor, or bootloader** →
  [Assembly for OS Development](../asm/00-overview.md) +
  [Systems From First Principles](../systems/00-overview.md) +
  [OxideOS](../oxideos/00-overview.md) as a full working reference.
- **Building a physical device** (a sensor node, a robot controller, a
  wearable, anything with real GPIO/I2C/SPI hardware and no need to run
  arbitrary other programs) → this section, starting from
  [Chapter 1](./01-no-std-and-the-embedded-toolchain.md).
- **Genuinely unsure which** → the MMU question is usually decisive: if
  the target chip has one and can run a real OS, you're closer to the
  first path; if it's a microcontroller that will only ever run your
  one program, you're on this one.
