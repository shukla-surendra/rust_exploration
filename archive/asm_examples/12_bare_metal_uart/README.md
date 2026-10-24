# Bare-Metal UART: Real Hardware Communication, No OS

The source behind
[`docs/src/asm/11-io-ports-and-mmio.md`](../../docs/src/asm/11-io-ports-and-mmio.md) —
read that chapter for the full explanation. This folder is the actual
code, kept here rather than only inline in the doc.

This is a genuinely different kind of example from everything else in
`asm_examples/`: no `main`, no libc, no OS underneath at all — these
boot **directly** in QEMU as the very first code to run on a virtual
machine, which is the only way to legally execute `in`/`out` port-I/O
instructions (blocked in ring 3 under any real OS) or demonstrate raw
MMIO the way a device driver would actually see it.

## `x86_64_multiboot.s` — Port-Mapped I/O — **verified working**

A 32-bit protected-mode Multiboot kernel that initializes a real 16550
UART via `in`/`out` to ports `0x3F8`–`0x3FD`, then prints through it.

```bash
docker exec asm-amd64 bash -c "cd /work/12_bare_metal_uart && \
  as --32 x86_64_multiboot.s -o k.o && \
  ld -m elf_i386 -T x86_64_multiboot.ld k.o -o k.elf"
qemu-system-x86_64 -kernel k.elf -serial mon:stdio -display none -no-reboot
```

Verified output:

```
Hello from bare metal, via real port I/O!
```

(Native macOS `as`/`ld` can't produce this — they don't support i386 ELF
output or GNU linker scripts. Build inside a Linux container, same as
every other Linux-targeting file in this collection.)

## `arm64_qemu_virt.s` — Memory-Mapped I/O — **verified working**

The MMIO equivalent, targeting the PL011 UART QEMU's `virt` machine maps
at `0x09000000`. Same protocol shape as the x86 file — a control-register
init, then a poll-before-write handshake — through loads/stores to a
fixed address instead of `in`/`out`.

```bash
docker exec asm-arm64 bash -c "cd /work/12_bare_metal_uart && \
  as arm64_qemu_virt.s -o k.o && \
  ld -T arm64_qemu_virt.ld k.o -o k.elf"
qemu-system-aarch64 -M virt -cpu cortex-a72 -nographic -kernel k.elf
```

Verified output:

```
Hello from bare metal, via real MMIO!
```

**This took two rounds of debugging, and the first fix was wrong** —
worth knowing since it's a genuinely common bare-metal trap. The
original draft (no `uart_init`) hung, and instruction-tracing
(`qemu -d in_asm`) showed it stuck polling the Flag Register's TXFF bit
— which looked exactly like "the UART was never enabled." Adding
`uart_init` (a correct PL011 init sequence) didn't fix it. The actual
bug: **`_start` never set `sp`** before the first function call pushed
onto it — an uninitialized stack pointer, not a UART configuration
issue. Confirmed by comparing against the real `hello-kernel` project
(`~/projects/hello-kernel`), which does the identical PL011 polling
with **no UART init at all** and works — proving `uart_init` was never
the real fix. `uart_init` stays in the file anyway (it's correct,
harmless, real hardware practice), but the three instructions that
actually mattered were setting `sp` at the very top of `_start`. Full
writeup, including the instruction trace, in
[`docs/src/asm/11-io-ports-and-mmio.md`](../../docs/src/asm/11-io-ports-and-mmio.md).

## Why there's no macOS-native build step for either file

Both are freestanding ELF binaries meant for QEMU, not the host OS
(macOS in this case). Building either one is exactly the same on macOS
or Linux — you need the Linux containers' GNU binutils either way, since
macOS's own `as`/`ld` target Mach-O and don't support `-m elf_i386` or
GNU linker scripts (`-T`) at all. *Running* them is trivial on macOS,
since QEMU itself runs natively — only the build step needs the
container.
