# 15. Debugging With LLDB & GDB

Every previous chapter proved a program worked by reading its final
output or exit code. A debugger lets you watch it work — pausing
execution at a chosen instruction and inspecting exactly what's in every
register and byte of memory at that moment. This is the tool that turns
"my program produced the wrong answer" into "register `x2` held 7 instead
of 8 at instruction `0x100000304`, because—", which is where every real
bug actually gets found.

**lldb** is Apple's debugger, ships with the macOS Command Line Tools
([Chapter 2](./02-toolchain-setup.md)), and is also what Rust's own
tooling uses by default on macOS. **gdb** is the traditional Linux
debugger, installed alongside `gcc`/`binutils` in Chapter 2. They solve
the same problem with mostly-equivalent commands under different names.

## The core workflow, either tool

1. Load the binary.
2. Set a **breakpoint** — an address (or symbol name) where execution
   should pause.
3. **Run** — the process executes at full speed until it hits that
   breakpoint.
4. Inspect: registers, memory, the call stack.
5. **Step** — advance by exactly one instruction, or resume to the next
   breakpoint, repeating from step 4.

## GDB (Linux)

```bash
as -g sum.s -o sum.o && ld sum.o -o sum      # -g embeds debug info
gdb ./sum
```

```
(gdb) break loop_top          # pause every time execution reaches this label
(gdb) run                       # start the program
(gdb) info registers rbx rcx      # inspect specific registers
(gdb) stepi                         # execute exactly one instruction
(gdb) continue                        # resume until the next breakpoint hit
(gdb) print $rbx                        # inspect a register directly
(gdb) x/4xb $rsi                          # examine memory: 4 hex bytes at the address in rsi
(gdb) disassemble                           # show the assembly around where you're stopped
```

Running [Chapter 9](./09-control-flow.md)'s sum-loop this way, you'd watch
`rbx` (the running sum) and `rcx` (the loop counter `i`) tick upward on
every `continue`, landing on `1`, then `3`, then `6`, then `10`... exactly
the running total the hand-trace in that chapter predicted — a debugger
is the tool that lets you *confirm* a hand-trace like that instead of just
trusting it.

## LLDB (macOS)

Same workflow, different command spelling:

```bash
as -g sum_mac.s -o sum_mac.o
ld sum_mac.o -o sum_mac -lSystem -syslibroot $(xcrun --show-sdk-path) \
   -e _main -arch arm64 -platform_version macos 11.0 11.0
codesign -s - sum_mac
lldb ./sum_mac
```

```
(lldb) breakpoint set --name loop_top     # or: b loop_top
(lldb) run
(lldb) register read x1 x2
(lldb) thread step-inst                     # execute exactly one instruction (si also works)
(lldb) continue
(lldb) print $x1
(lldb) memory read --size 1 --count 4 $x1
(lldb) disassemble
```

## GDB ↔ LLDB, side by side

| Action | GDB | LLDB |
|---|---|---|
| Set breakpoint at a label | `break loop_top` | `b loop_top` (short for `breakpoint set --name loop_top`) |
| Run | `run` | `run` |
| Step one instruction | `stepi` | `thread step-inst` (or `si`) |
| Continue to next breakpoint | `continue` | `continue` (or `c`) |
| Read registers | `info registers` | `register read` |
| Read one register | `print $rax` | `print $x0` |
| Examine memory | `x/4xb $rsi` | `memory read --size 1 --count 4 $x1` |
| Disassemble current location | `disassemble` | `disassemble` |
| Print the call stack | `backtrace` | `bt` |

## If breakpoints don't seem to stop your program

Worth naming directly, since it's a real thing you may hit: some
sandboxed shells, CI runners, and restricted containers **block the
underlying OS mechanism a debugger needs to pause another process**
(`ptrace` on Linux, `task_for_pid` on macOS) — even though the debugger
launches your program successfully and reports its exit code, breakpoints
silently never trigger and it just runs straight through to completion.
This isn't a bug in your assembly or your breakpoint syntax — if you're in
a Docker container, add `--cap-add=SYS_PTRACE`; if that's still not
enough, the container runtime's default seccomp profile may need
`--security-opt seccomp=unconfined` as well. On a plain terminal on your
own Mac or Linux machine (not nested inside another sandbox), this
essentially never comes up.

## Reading a crash: the one thing worth memorizing this chapter for

You will eventually write assembly (or trigger a bug in a language whose
compiler generated bad-but-legal assembly) that segfaults. When it does,
run it under the debugger and the very first thing to check is:

```
(gdb) info registers rip     # or, in lldb: register read pc
```

The instruction pointer/program counter at the moment of the crash tells
you *exactly* which instruction faulted — cross-reference that address
against `disassemble` (or the `objdump`/`otool` output from
[Chapter 14](./14-reading-compiler-output.md)) and you know precisely
which line of assembly (and, with debug info, which line of source) was
executing. Then `info registers` (or `register read`) shows you every
value involved — almost always enough, on its own, to see the bad
address or bad value that caused it.

## What's next

You can now read compiled assembly and watch it execute live.
[Chapter 16](./16-cheat-sheet.md) is the reference you'll actually keep
open in a tab going forward — every instruction, register, and syscall
convention from this whole section, side by side, one page.
