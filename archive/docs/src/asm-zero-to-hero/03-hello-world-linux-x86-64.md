# 3. Hello, World — Linux x86-64

The full program, verified working (built and run inside the `asm-amd64`
Docker container set up in [Chapter 2](./02-toolchain-setup.md)):

```asm
.global _start
.section .text
_start:
    mov $1, %rax        # syscall number 1 = write
    mov $1, %rdi         # 1st arg: file descriptor 1 = stdout
    lea msg(%rip), %rsi  # 2nd arg: address of the text to print
    mov $14, %rdx         # 3rd arg: how many bytes to print
    syscall

    mov $60, %rax        # syscall number 60 = exit
    xor %rdi, %rdi         # 1st arg: exit code 0
    syscall

.section .data
msg:
    .ascii "Hello, world!\n"
```

Save as `hello.s`, then:

```bash
docker exec asm-amd64 bash -c "cd /work && as hello.s -o hello.o && ld hello.o -o hello && ./hello"
```

Output:

```
Hello, world!
```

## Line by line, assuming zero prior knowledge

**`.global _start`** — a directive (starts with `.`, an instruction *to
the assembler*, not to the CPU) saying "the symbol `_start` should be
visible outside this file." The linker needs this because it's about to
go looking for exactly one symbol named `_start` to know where your
program should begin running — on Linux, `_start` is the conventional,
required name for that entry point.

**`.section .text`** — another assembler directive. An executable file is
divided into named regions; `.text` is the conventional name for "this is
where code (instructions) lives." (You'll see `.data` below, for values
instead of instructions — mixing the two is legal but bad practice, one
more reason they're kept in separate sections.)

**`_start:`** — a **label**. It doesn't produce any machine code by
itself; it just gives a name to "the address of whatever instruction comes
right after this line," so other lines (or the linker) can refer to that
address by name instead of a raw number.

**`mov $1, %rax`** — `mov` copies a value into a register (it does *not*
"move" in the sense of removing it from the source — the name is
historically misleading; think "copy"). Recall from
[Chapter 2](./02-toolchain-setup.md#which-assembly-syntax-this-section-uses):
AT&T syntax is *source, then destination*, so this reads "copy the value 1
into register `rax`." `$1` is an **immediate** — a literal number baked
directly into the instruction, marked with `$` in AT&T syntax so the
assembler doesn't confuse it with a register name or a memory address.
`%rax` is a **register** — one of the CPU's own tiny storage slots (full
explanation in [Chapter 6](./06-registers-plain-english.md)); the `%`
prefix is, again, an AT&T-syntax thing, marking "this is a register name."

**Why `rax` specifically, and why `1`**: this is the crux of a syscall.
You are not calling a function *in your own program* — you're asking the
**Linux kernel** to do something on your behalf (writing to a screen,
reading a file, exiting a process — things your program isn't allowed to
do directly, since the kernel guards all of that). The way you ask is a
strict, documented convention: put a **syscall number** in `rax` first (1
happens to mean "write" on Linux x86-64 — this exact number is what
[Chapter 12](./12-syscalls-deep-dive.md) covers in full, including why it
differs on macOS), then put that syscall's arguments in a fixed set of
other registers, then execute the single instruction `syscall`, which
hands control to the kernel. The kernel reads those same registers, does
the work, and hands control back.

**`mov $1, %rdi`** — the *first argument* to the `write` syscall: which
**file descriptor** to write to. `1` is the OS's standing convention for
"standard output" (your terminal) — this isn't an assembly rule, it's an
operating-system-wide convention that C's `printf`, Python's `print`,
everything, ultimately relies on too.

**`lea msg(%rip), %rsi`** — the *second argument*: **where** the text to
print lives in memory. `lea` means "load effective address" — unlike
`mov`, it doesn't read the value stored at an address, it computes an
address and puts *that number* into the register. `msg` is the label
defined near the bottom, pointing at the text. `(%rip)` means "relative to
the current instruction pointer" — a **RIP-relative** address, the modern,
default way x86-64 code refers to its own data; it works correctly no
matter where in memory the OS decides to load your program (this matters
more once shared libraries and address-space randomization enter the
picture — for now, just know it's the standard, safe way to say "the
address of `msg`").

**`mov $14, %rdx`** — the *third argument*: how many bytes to print.
`"Hello, world!\n"` is exactly 14 characters (13 letters/punctuation plus
the newline `\n`) — this number has to be counted by hand here, which is
exactly the kind of tedious, error-prone bookkeeping
[Chapter 13](./13-arrays-strings-algorithms.md) shows you how to compute
automatically instead.

**`syscall`** — the actual instruction that traps into the kernel. Every
register you set up above is what the kernel reads to know what to do.

**`mov $60, %rax` / `xor %rdi, %rdi` / `syscall`** — a second syscall: 60
is `exit` on Linux x86-64. `xor %rdi, %rdi` is a common idiom meaning
"set `rdi` to zero" — XOR-ing any value with itself always produces zero,
and this is conventionally preferred over `mov $0, %rdi` because the
resulting machine-code instruction is shorter (one fewer byte). `rdi`
here is `exit`'s one argument: the process's **exit code**, the number a
shell sees as `$?` after the program finishes.

**`.section .data` / `msg: .ascii "Hello, world!\n"`** — switches to the
data section, then `.ascii` is a directive telling the assembler "place
these literal bytes here" (as opposed to interpreting them as
instructions). `msg` labels the address of the first of those bytes,
which is exactly the address `lea msg(%rip), %rsi` loaded above.

## What actually happens when you run it

1. The OS loads your program into memory and jumps to `_start`.
2. Registers get loaded: syscall number 1, fd 1, address of `msg`, length
   14.
3. `syscall` — control passes to the kernel, which writes those 14 bytes
   to your terminal.
4. Control returns to your program, right after the first `syscall`.
5. Registers get loaded again: syscall number 60, exit code 0.
6. `syscall` — the kernel terminates your process. Nothing after this
   line ever executes; there's nothing after it anyway.

## Try it yourself

Change `$0` to `$0` → `$42` on the exit line (well, `xor %rdi,%rdi` sets 0
— replace that whole line with `mov $42, %rdi`), rebuild, run, then check:

```bash
docker exec asm-amd64 bash -c "cd /work && ./hello; echo \$?"
```

You should see `42`. This is the single fastest way to confirm a change
you made actually took effect — no `printf` needed, just watch the exit
code.

Next: the same program, same operating system, different CPU —
[Chapter 4](./04-hello-world-linux-arm64.md).
