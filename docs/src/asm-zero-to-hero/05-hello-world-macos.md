# 5. Hello, World — macOS (Both Architectures)

Same ARM64 instruction set as [Chapter 4](./04-hello-world-linux-arm64.md)
— but a different OS underneath changes the syscall convention, the
entry-point name, the file format, and adds a step Linux never requires:
**code signing**. This chapter also builds the x86-64 macOS version, run
via Rosetta 2 — every example below was actually assembled, linked, and
run natively on Apple Silicon macOS.

## ARM64 macOS (Apple Silicon, native)

```asm
.global _main
.align 4
.section __TEXT,__text
_main:
    mov x0, #1              // 1st arg: file descriptor 1 = stdout
    adrp x1, msg@PAGE       // 2nd arg: address of the text (page part)
    add x1, x1, msg@PAGEOFF //          ...plus the offset within that page
    mov x2, #14              // 3rd arg: how many bytes to print
    mov x16, #4               // syscall number 4 = write (BSD numbering)
    svc #0x80

    mov x0, #0
    mov x16, #1                // syscall number 1 = exit
    svc #0x80

.section __DATA,__data
msg:
    .ascii "Hello, world!\n"
```

```bash
as hello_mac.s -o hello_mac.o
ld hello_mac.o -o hello_macos_arm64 \
   -lSystem -syslibroot $(xcrun --show-sdk-path) \
   -e _main -arch arm64 -platform_version macos 11.0 11.0
codesign -s - hello_macos_arm64
./hello_macos_arm64
```

```
Hello, world!
```

## Every difference from Linux ARM64, explained

**Entry point is `_main`, not `_start`, and it needs `-e _main` at link
time**: macOS's convention (inherited from the wider BSD/Mach tradition
Darwin comes from) is that the linker expects `_main` unless told
otherwise; `-e _main` on the `ld` line makes that explicit.

**Sections are named `__TEXT,__text` / `__DATA,__data`, not `.text` /
`.data`**: this isn't stylistic — it's because macOS uses the **Mach-O**
executable format, not Linux's ELF, and Mach-O groups sections under
named **segments** (`__TEXT`, `__DATA`) each with their own memory
permissions and their own named sub-sections inside.

**`-lSystem -syslibroot ...` is mandatory**: unlike Linux, where a
`_start` that only makes raw syscalls can link with *nothing* else, macOS
executables must link against `libSystem` (Apple's umbrella library
covering libc and more) even if you never call a single C function — the
dynamic linker (`dyld`) simply refuses to run a Mach-O binary that isn't
linked against it. `-syslibroot $(xcrun --show-sdk-path)` points the
linker at Apple's SDK, where `libSystem` actually lives.

**`codesign -s - hello_macos_arm64` is not optional on Apple Silicon.**
Skip it and you'll see `zsh: killed` or `Bad CPU type in executable`
the instant you try to run it — Apple Silicon macOS enforces that every
executable carries a valid code signature before it's allowed to run at
all, even one built entirely locally with no intent to distribute it.
`-s -` requests an **ad-hoc signature** — self-signed, no Apple Developer
account or certificate needed, sufficient purely to satisfy this local
execution requirement.

**`mov x16, #4` / `mov x16, #1`, not `x8`**: the syscall number goes in a
*different register* than on Linux ARM64 — `x16` instead of `x8`. This is
purely an OS convention difference, nothing to do with the CPU (the CPU
doesn't care which register holds what; it's the kernel on the other side
of `svc` that decides which register it reads).

**Syscall numbers 4 and 1, not 64 and 93**: macOS's syscall table
descends from BSD Unix, a completely separate lineage from Linux's table
— 4 happens to be `write` and 1 happens to be `exit` on Darwin/BSD, with
no relationship to Linux's numbers for the same operations. (You'll also
sometimes see these BSD syscall numbers with `0x2000000` added — that's
specifically an **x86-64 macOS** encoding detail, covered below; ARM64
macOS does not need that offset, exactly as shown here.)

**`svc #0x80`, not `svc #0`**: same "supervisor call" instruction as
Linux ARM64, but with the immediate operand set to `0x80` by convention —
this is inherited directly from the equivalent `int 0x80` interrupt
number BSD/x86 systems have used historically, carried over to ARM64 macOS
even though ARM has no literal "interrupt 0x80" concept — it's the
number's *meaning as a convention* that survived, not any hardware
requirement to use it.

**`adrp ... @PAGE` / `add ... @PAGEOFF`, not a single `adr`**: functionally
the same job as Chapter 4's `adr x1, msg` (compute the address of `msg`),
but expressed differently because Mach-O's relocation system doesn't
support the single-instruction `adr` relocation the same way ELF does for
this case — `@PAGE`/`@PAGEOFF` is Apple's assembler syntax for the
two-instruction "address of the containing page, then add the offset
within it" pattern, functionally the same idea as the `:lo12:` syntax used
in [Chapter 8](./08-memory-and-addressing.md) and in this book's
[OS-development assembly section](../asm/02-registers-and-calling-conventions.md#adrplo12-the-aarch64-address-loading-idiom)
for the same underlying instruction-encoding limitation (a 4-byte ARM64
instruction can't hold an arbitrary 64-bit address).

## x86-64 macOS (via Rosetta 2, on Apple Silicon)

If you're on an Intel Mac, this runs natively — no Rosetta needed. On
Apple Silicon, you need Rosetta installed
([Chapter 2](./02-toolchain-setup.md)) to *execute* it; building it
requires nothing extra, since `clang` can cross-compile:

```asm
.global _main
.section __TEXT,__text
_main:
    mov $0x2000004, %rax   # syscall 4 (write) + 0x2000000 BSD-class offset
    mov $1, %rdi
    lea msg(%rip), %rsi
    mov $14, %rdx
    syscall

    mov $0x2000001, %rax   # syscall 1 (exit) + 0x2000000 BSD-class offset
    xor %rdi, %rdi
    syscall

.section __DATA,__data
msg:
    .ascii "Hello, world!\n"
```

```bash
clang -target x86_64-apple-macos11 -c hello_mac_x86.s -o hello_mac_x86.o
ld hello_mac_x86.o -o hello_macos_x86_64 \
   -lSystem -syslibroot $(xcrun --show-sdk-path) \
   -e _main -arch x86_64 -platform_version macos 11.0 11.0
codesign -s - hello_macos_x86_64
arch -x86_64 ./hello_macos_x86_64
```

```
Hello, world!
```

The instructions themselves (`mov`, `lea`, `syscall`) are **word-for-word
identical to Linux x86-64** in Chapter 3 — same CPU, same instruction set,
zero differences there. Everything different is the macOS layer on top:
`_main` instead of `_start`, `__TEXT`/`__DATA` sections, the `-lSystem`
link requirement, code signing, and — the one true x86-64-macOS-specific
quirk — **syscall numbers need `0x2000000` added** to the plain BSD number
(so `write` becomes `0x2000004`, not `4`). This offset is Apple's way of
tagging "this is a Unix/BSD-class syscall" within a wider syscall-class
scheme XNU (the Darwin kernel) supports; ARM64 macOS doesn't need you to
add it explicitly because `svc #0x80`'s meaning already implies that class.

`arch -x86_64 ./binary` explicitly forces execution through Rosetta 2's
translation layer even on an Apple Silicon Mac that could otherwise run
the native ARM64 build directly — this is the command that actually proves
Rosetta is doing real work here, translating x86-64 machine code to ARM64
on the fly.

## The full four-way comparison, now that all four exist

| | Linux x86-64 | Linux ARM64 | macOS ARM64 | macOS x86-64 |
|---|---|---|---|---|
| Entry symbol | `_start` | `_start` | `_main` | `_main` |
| File format | ELF | ELF | Mach-O | Mach-O |
| Section names | `.text`/`.data` | `.text`/`.data` | `__TEXT,__text`/`__DATA,__data` | `__TEXT,__text`/`__DATA,__data` |
| Trap instruction | `syscall` | `svc #0` | `svc #0x80` | `syscall` |
| Syscall # register | `rax` | `x8` | `x16` | `rax` |
| `write` syscall # | 1 | 64 | 4 | `0x2000004` |
| `exit` syscall # | 60 | 93 | 1 | `0x2000001` |
| Needs `-lSystem`? | no | no | yes | yes |
| Needs code signing? | no | no | yes | yes (ad-hoc suffices) |

Every cell in that table was independently verified while writing this
chapter, not assumed from documentation.

Next: with four working hello-worlds behind you, it's time to properly
understand the thing every one of them leaned on without explanation —
registers — in [Chapter 6](./06-registers-plain-english.md).
