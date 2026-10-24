# 2. Setting Up Your Toolchain

Three tools, every platform: an **assembler** (turns your `.s` file into
machine code), a **linker** (stitches machine code + libraries into a
runnable executable), and a **debugger** (lets you step through it one
instruction at a time). This chapter gets all three working on macOS and
on Linux, plus a way to test *both* Linux architectures from a single Mac.

## macOS (Apple Silicon or Intel)

Everything needed ships with Apple's **Command Line Tools** — no
Homebrew required for the core workflow:

```bash
xcode-select --install
```

This gives you:

| Tool | What it is |
|---|---|
| `as` | The assembler (Apple's, built on the LLVM assembler) |
| `ld` | The linker |
| `clang` | Can also assemble+link in one step, and cross-compile — used from Chapter 5 onward |
| `lldb` | The debugger — [Chapter 15](./15-debugging.md) |
| `codesign` | **Required on Apple Silicon.** macOS refuses to run an ARM64 binary that isn't code-signed, even a trivial hand-written one. An *ad-hoc* signature (no Apple Developer account needed) is enough: `codesign -s - your_binary` |
| `otool -tv` | Disassembler — shows you the machine code Apple's own tools produced, handy for comparing against what you wrote by hand |

Sanity check:

```bash
as --version
ld -version
clang --version
```

If you're on an **Intel Mac**, you already have a native x86-64 toolchain
and can skip straight to [Chapter 3](./03-hello-world-linux-x86-64.md)'s
sibling in [Chapter 5](./05-hello-world-macos.md). If you're on **Apple
Silicon** wanting to also assemble x86-64 macOS binaries (as this section
does, purely to show the contrast), you need **Rosetta 2** installed to
*run* them:

```bash
softwareupdate --install-rosetta --agree-to-license
```

`clang -target x86_64-apple-macos11` (covered in Chapter 5) can still
*build* an x86-64 Mach-O binary without Rosetta — Rosetta is only needed
to actually execute it, via `arch -x86_64 ./your_binary`.

## Linux (native, or a Linux VM/box you already have)

The equivalent toolchain comes from **binutils** (assembler + linker),
**gcc** (used here purely as a convenient assemble+link driver and for
[Chapter 11](./11-calling-c-from-asm.md)'s C interop), and **gdb**:

```bash
# Debian/Ubuntu
sudo apt-get update && sudo apt-get install -y gcc binutils gdb

# Fedora/RHEL
sudo dnf install -y gcc binutils gdb

# Arch
sudo pacman -S gcc binutils gdb
```

Sanity check:

```bash
as --version
ld --version
gdb --version
```

No code-signing step on Linux — a freshly linked executable with the
execute bit set (`ld` sets this automatically) just runs.

## Testing Linux from a Mac: the Docker trick

You don't own a Linux ARM64 server and an Intel Linux box just to follow
along — and you don't need to. **Every Linux example in this section was
actually built and run this way**, using Docker's `--platform` flag to
pick the CPU architecture *independent of your host Mac's own
architecture* (Docker Desktop on Apple Silicon can run both `amd64` and
`arm64` Linux containers via emulation/native support):

```bash
# One-time: pull and prep a reusable container per architecture
docker run -d --name asm-amd64 --platform linux/amd64 \
  -v "$(pwd)":/work -w /work ubuntu:22.04 sleep infinity
docker run -d --name asm-arm64 --platform linux/arm64 \
  -v "$(pwd)":/work -w /work ubuntu:22.04 sleep infinity

docker exec asm-amd64 bash -c "apt-get update -qq && apt-get install -y -qq gcc binutils gdb"
docker exec asm-arm64 bash -c "apt-get update -qq && apt-get install -y -qq gcc binutils gdb"
```

`-v "$(pwd)":/work` mounts your current directory straight into the
container, so you edit `.s` files normally in your Mac's editor and just
run the assemble/link/run commands *inside* the container:

```bash
docker exec asm-amd64 bash -c "cd /work && as hello.s -o hello.o && ld hello.o -o hello && ./hello"
```

Repeat with `asm-arm64` for the other architecture. This is exactly how
every Linux example in Chapters 3, 4, and onward was verified — two
long-lived containers, one per architecture, reused for the whole section
instead of re-provisioning per example.

## Which assembly *syntax* this section uses

Two unrelated choices exist and it's worth naming them up front so
nothing looks arbitrarily different between chapters:

- **x86-64 has two competing syntaxes**: **AT&T** (source, then
  destination — `mov $5, %rax`, registers prefixed with `%`, immediates
  with `$`) and **Intel** (destination, then source — `mov rax, 5`, no
  prefixes). **This section uses AT&T**, because it's what the default
  GNU assembler (`as`, on both Linux and macOS) and GCC/Clang's inline
  assembly expect without extra flags. Chapter 16's cheat sheet includes a
  side-by-side so you can translate at sight if you encounter Intel syntax
  elsewhere (most online x86 tutorials and Windows-focused material use
  Intel syntax — it's not that AT&T is more "correct," just what this
  toolchain defaults to).
- **ARM64 has one standard syntax** (no AT&T/Intel-style split) — what's
  shown here is it, unmodified.

## Quick end-to-end smoke test

Before moving on, confirm every environment actually works with the
smallest possible program — one instruction, immediately exits:

```bash
# macOS ARM64 (native)
echo '.global _main
.section __TEXT,__text
_main:
    mov x0, #7
    mov x16, #1
    svc #0x80' > smoke.s
as smoke.s -o smoke.o
ld smoke.o -o smoke -lSystem -syslibroot $(xcrun --show-sdk-path) -e _main -arch arm64 -platform_version macos 11.0 11.0
codesign -s - smoke
./smoke; echo "exit code: $?"   # expect 7
```

```bash
# Linux x86-64 (in the amd64 container)
echo '.global _start
.section .text
_start:
    mov $7, %rdi
    mov $60, %rax
    syscall' > smoke.s
docker exec asm-amd64 bash -c "cd /work && as smoke.s -o smoke.o && ld smoke.o -o smoke && ./smoke; echo \"exit code: \$?\""   # expect 7
```

If both print `exit code: 7`, every tool is wired up correctly and you're
ready for [Chapter 3](./03-hello-world-linux-x86-64.md).
