# 16. x86-64 vs ARM64 Cheat Sheet

The one page to keep open in a tab. Every fact here was established, with
reasoning, somewhere in Chapters 1–15 — this is the lookup table, not the
explanation; follow the chapter links for the "why."

## Syntax basics

| | x86-64 (AT&T, this section's choice) | x86-64 (Intel, common elsewhere) | ARM64 |
|---|---|---|---|
| Operand order | source, dest | dest, source | dest, source |
| Register prefix | `%rax` | `rax` | none: `x0` |
| Immediate prefix | `$5` | `5` | `#5` |
| Memory deref | `(%rax)` | `[rax]` | `[x0]` |
| Example: `dst = src` | `mov %rbx, %rax` | `mov rax, rbx` | `mov x0, x1` |

## Registers

| Role | x86-64 | ARM64 |
|---|---|---|
| General purpose (count) | 16: `rax`,`rbx`,`rcx`,`rdx`,`rsi`,`rdi`,`rbp`,`rsp`,`r8`–`r15` | 31: `x0`–`x30` |
| Zero register | none — use `xor reg,reg` | `xzr`/`wzr` |
| Stack pointer | `rsp` | `sp` (16-byte aligned) |
| Frame pointer | `rbp` (conventional) | `x29` / `fp` |
| Return address | pushed to stack by `call`, popped by `ret` | held in `x30` / `lr` by `bl` |
| Program counter | `rip` (indirect access only) | `pc` (indirect access only) |
| 32-bit view of a register | `eax` (bottom half of `rax`) | `w0` (bottom half of `x0`) |
| 8-bit view | `al` (bottom byte of `rax`) | — (use `w0` and mask, or byte load/store) |

Full explanation: [Chapter 6](./06-registers-plain-english.md).

## Instructions

| Operation | x86-64 | ARM64 |
|---|---|---|
| Copy | `mov src, dst` | `mov dst, src` |
| Load from memory | `mov (%r), %r2` (fused with move) | `ldr r2, [r]` (always separate) |
| Store to memory | `mov %r2, (%r)` | `str r2, [r]` |
| Add | `add src, dst` (2-operand, dst=dst+src) | `add dst, a, b` (3-operand, dst=a+b) |
| Subtract | `sub src, dst` | `sub dst, a, b` |
| Multiply (signed) | `imul src, dst` | `mul dst, a, b` |
| Divide (signed) | `idiv src` (uses `rax`/`rdx`) | `sdiv dst, a, b` |
| Increment/decrement | `inc %r` / `dec %r` | no dedicated form — `add r,r,#1` / `sub r,r,#1` |
| Compare | `cmp src, dst` (computes dst-src) | `cmp a, b` (computes a-b) |
| Compute an address (no memory access) | `lea offset(%r), %dst` | `adr dst, label` (near) / `adrp`+`add` (far) |
| Zero a register | `xor %r, %r` | `mov r, xzr` (or just `mov r, #0`) |
| Function call | `call target` (pushes return addr) | `bl target` (sets `x30`/`lr`) |
| Return | `ret` (pops return addr) | `ret` (jumps to `x30`/`lr`) |
| Push/pop | `push %r` / `pop %r` | no dedicated form — `str`/`ldr` with `sp`, or `stp`/`ldp` for pairs |
| Unconditional jump | `jmp label` | `b label` |

Full explanation: [Chapter 7](./07-instructions-101.md).

## Conditional jumps (after a `cmp`)

| Meaning | x86-64 | ARM64 |
|---|---|---|
| equal | `je` | `b.eq` |
| not equal | `jne` | `b.ne` |
| less than (signed) | `jl` | `b.lt` |
| less or equal (signed) | `jle` | `b.le` |
| greater than (signed) | `jg` | `b.gt` |
| greater or equal (signed) | `jge` | `b.ge` |
| below (unsigned) | `jb` | `b.lo` |
| above (unsigned) | `ja` | `b.hi` |

Full explanation: [Chapter 9](./09-control-flow.md).

## Calling convention (ordinary function calls)

| | x86-64 System V (Linux + macOS) | ARM64 AAPCS64 (Linux + macOS, mostly) |
|---|---|---|
| Args 1–6 | `rdi`,`rsi`,`rdx`,`rcx`,`r8`,`r9` | `x0`–`x7` |
| Return value | `rax` | `x0` |
| Caller-saved | `rax`,`rcx`,`rdx`,`rsi`,`rdi`,`r8`–`r11` | `x0`–`x18` |
| Callee-saved | `rbx`,`rbp`,`r12`–`r15` | `x19`–`x28`,`x29`,`x30` |
| Stack alignment at call | 16 bytes | 16 bytes |
| **Variadic args (e.g. `printf`)** | extra args in registers; `al` = float-reg count | **Linux: registers, standard. macOS: stack — a real Apple ABI deviation, see [Chapter 11](./11-calling-c-from-asm.md)** |

## Syscalls

| | Linux x86-64 | Linux ARM64 | macOS x86-64 | macOS ARM64 |
|---|---|---|---|---|
| Trap instruction | `syscall` | `svc #0` | `syscall` | `svc #0x80` |
| Number register | `rax` | `x8` | `rax` | `x16` |
| Args 1–4 | `rdi`,`rsi`,`rdx`,`r10` | `x0`–`x3` | `rdi`,`rsi`,`rdx`,`r10` | `x0`–`x3` |
| `write` | 1 | 64 | `0x2000004` | 4 |
| `exit` | 60 | 93 | `0x2000001` | 1 |
| `read` | 0 | 63 | `0x2000003` | 3 |

Full explanation: [Chapter 12](./12-syscalls-deep-dive.md). Numbers not
listed here: check `/usr/include/asm-generic/unistd.h` on Linux, or
`/usr/include/sys/syscall.h` on macOS.

## Platform/toolchain quick reference

| | Linux x86-64 | Linux ARM64 | macOS x86-64 | macOS ARM64 |
|---|---|---|---|---|
| Entry symbol | `_start` | `_start` | `_main` | `_main` |
| File format | ELF | ELF | Mach-O | Mach-O |
| Text section | `.text` | `.text` | `__TEXT,__text` | `__TEXT,__text` |
| Data section | `.data` | `.data` | `__DATA,__data` | `__DATA,__data` |
| Needs `-lSystem`? | no | no | yes | yes |
| Needs code signing? | no | no | ad-hoc suffices | **required**, even ad-hoc |
| Disassembler | `objdump -d` | `objdump -d` | `otool -tv` | `otool -tv` |
| Debugger | `gdb` | `gdb` | `lldb` | `lldb` |

## What's next

The reference is done — [Chapter 17](./17-mini-projects.md) is where all
of it gets used together, building three complete small programs, each
run for real on multiple platforms.
