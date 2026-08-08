# 10. `hello-kernel`: UART Output & Panics

> Continues the [`hello-kernel` case study](./07-hello-kernel-overview.md)
> — imported from a separate project, not part of this repo.

## UART output — how the string actually appears on screen

The `virt` machine's PL011 UART is memory-mapped at `0x0900_0000` — a
device register, not RAM, but addressed the same way RAM is (chapter 1's
"an address is just a number identifying which byte you mean" applies
identically here, even though writing to this address transmits a byte
instead of storing one):

- `UART_DR` (offset `0x00`) — data register; writing a byte here
  transmits it.
- `UART_FR` (offset `0x18`) — flag register; bit 5 (`TXFF`) is set while
  the transmit FIFO is full.

```rust
fn uart_putc(c: u8) {
    unsafe {
        while core::ptr::read_volatile(UART_FR as *const u32) & UART_FR_TXFF != 0 {}
        core::ptr::write_volatile(UART_DR as *mut u32, c as u32);
    }
}
```

For each byte: busy-wait while the FIFO is full, then write the byte.
`read_volatile`/`write_volatile` are required because these are
hardware registers with side effects — the compiler must not reorder,
cache, or elide these accesses the way it legitimately could with
ordinary memory. An optimizing compiler is normally free to assume "if I
already read this value and nothing in my code changed it, I can reuse
the old value instead of reading again" — exactly the assumption that
would break here, since the *hardware* changes `UART_FR` out from under
the program as the FIFO drains. `volatile` is the escape hatch: "read
this from the real address, every single time, no matter what the
compiler thinks it already knows."

`uart_puts` just calls `uart_putc` once per byte of a `&str` — the
[`&str`/byte iteration](../foundation/strings.md) is ordinary Rust; only
the actual register access needed `unsafe`/`volatile`. QEMU's PL011
model forwards every byte written to `UART_DR` straight to your
terminal (because of `-nographic`, from chapter 9), which is the whole
reason `Hello, kernel!` shows up in your shell at all — there's no
"printing" in the OS-facilities sense anywhere in this program, just
raw bytes hitting a hardware register.

## Steady state and panics

After printing, the core sits in the `wfe` loop from chapter 9 — the
kernel is finished and idles forever. QEMU keeps running; quit with
`Ctrl-A X` (or `pkill qemu-system-aarch64` from another shell).

If anything panics before reaching that point, `#[panic_handler]` takes
over:

```rust
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    uart_puts("panic!\n");
    loop {
        unsafe { asm!("wfe") };
    }
}
```

`#![no_std]` binaries must supply their own panic handler — there's no
standard-library one to link against (unlike ordinary Rust code, where
a panic message and unwind/abort behavior come from `std` for free; see
[`todo!()`, `unimplemented!()`, `panic!()`](../foundation/error-handling.md#a-related-but-different-failure-mode-todo-unimplemented-panic)
for that normal-Rust picture). This handler reuses the same UART routine
to report the panic, then halts the same way `kernel_main` does — and
because `[profile.dev]`/`[profile.release]` both set `panic = "abort"`
(chapter 8), reaching this function never attempts stack unwinding, just
calls straight into it and halts.

## End-to-end summary

```
cargo run --release
  → cross-compile for aarch64-unknown-none, link with linker.ld
  → qemu-system-aarch64 -kernel <elf>
      → QEMU maps RAM at 0x40000000, UART at 0x09000000
      → CPU PC set to _start (0x40080000)
        → _start sets sp = __stack_top, branches to kernel_main
          → kernel_main writes "Hello, kernel!\n" byte-by-byte to UART_DR
          → kernel_main loops on `wfe` forever
```

Every arrow in that chain is a chapter in this case study: cross-compile
and link → [chapter 8](./08-hello-kernel-build-and-linking.md); QEMU
loading and jumping in, and why no separate bootloader was needed →
[chapter 9](./09-hello-kernel-boot-to-execution.md); the UART write loop
and panic handling → this chapter. Nothing in this ~50-line program is
unexplained — that completeness, on a real running system rather than
just prose, is the point of including it in this section at all.
