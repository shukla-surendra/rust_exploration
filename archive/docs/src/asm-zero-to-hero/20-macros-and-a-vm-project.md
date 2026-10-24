# 20. Assembler Macros, Flags in Depth, and a Capstone: Building a Tiny VM

[Chapter 18](./18-where-to-go-next.md) named two things this section
deliberately left out: **assembler macros / `.if`/`.endif` conditional
assembly** ("real tools for larger assembly projects, left out because
nothing in this section's examples was large enough to need them") and
**Windows** as a fifth environment. This chapter closes the first gap
directly, using this section's own toolchain (GAS, AT&T syntax, the same
Linux/macOS x86-64 environments as every earlier chapter) rather than
introducing a new assembler — and along the way builds something "large
enough to need" macros: a tiny bytecode virtual machine, hand-assembled,
with a real fetch-decode-execute dispatch loop.

The occasion for this chapter was reading gpfault.net's ["Let's Learn
x86-64 Assembly!"](https://gpfault.net/posts/asm-tut-0.txt.html) series,
which builds toward exactly this kind of project using a different
assembler (**FASM**) on a different OS (**Windows**, targeting the PE64
executable format, debugged with WinDbg). If Windows/PE internals and
FASM's own (quite different, `forward`/`reverse`/`match`-based) macro
system are of interest, that series is a well-written, thorough deep dive
in its own right and worth reading directly — it goes considerably further
into PE import-table structure and the Microsoft x64 calling convention
than this chapter does. Everything below is written and independently
verified against this book's own environment, not copied from there — same
underlying CPU concepts, different assembler, different OS, different
code.

Two things get filled in before the capstone project, because the VM
needs them and this section under-covered them so far: the flags register
in depth (why `mul`/`imul`/`div`/`idiv` behave the way they do), and the
*full* family of conditional jumps (`ja`/`jg`/`cmov`, not just the
`jz`/`jnz` [Chapter 9](./09-control-flow.md) already covered).

## The flags register, properly

[Chapter 6](./06-registers-plain-english.md) mentioned `rflags` only in
passing. Four bits in it matter for everything below:

| Flag | Set when | Used by |
|---|---|---|
| **ZF** (zero) | the result was exactly zero | `jz`/`jnz`, equality checks |
| **SF** (sign) | the result's high bit is 1 (i.e. negative, if you're treating it as signed) | signed comparisons |
| **CF** (carry) | an add/subtract needed (or borrowed) a bit beyond the operand width | **unsigned** comparisons |
| **OF** (overflow) | a *signed* add/subtract produced a result that doesn't fit back into a signed value of that width | **signed** comparisons |

The reason there are two separate "did this go wrong" flags (CF and OF)
instead of one is that the CPU doesn't know whether you meant a byte/word/
dword as signed or unsigned — it computes both flags on every arithmetic
instruction, and it's *your* choice of which flag (or combination) to test
afterward that decides which interpretation you get. This is also why
two's complement is the representation every mainstream CPU uses for
signed integers instead of a dedicated sign bit: negating a number is just
"flip every bit, then add one," which means the exact same `add`/`sub`
circuitry produces a correct result whether you treat the bits as signed
or unsigned — only the *flags you check afterward* differ.

## `mul` / `imul` / `div` / `idiv`: why the result lands in two registers

[Chapter 7](./07-instructions-101.md) introduced `mul`/`imul` briefly.
Here's the part that matters once you actually rely on them: multiplying
two N-bit numbers can produce a result needing up to 2N bits (`0xFF *
0xFF` doesn't fit in 8 bits), so these instructions always write a
double-width result, split across two registers:

| Operand width | `mul`/`imul` (1-operand form) reads | Result lands in |
|---|---|---|
| 8-bit | `al` | `ax` (high half in `ah`) |
| 16-bit | `ax` | `dx:ax` |
| 32-bit | `eax` | `edx:eax` |
| 64-bit | `rax` | `rdx:rax` |

`div`/`idiv` run the same idea backwards: the *dividend* has to already be
spread across that same register pair before you divide, and you get back
both a quotient (in the low register) and a remainder (in the high one) —
which is why a lone `div %ecx` right after `mov $47, %eax` is a bug unless
you also cleared `edx` first (`xor %edx, %edx`) — otherwise whatever
garbage was sitting in `edx` becomes the high half of the number you're
actually dividing.

Verified (macOS x86-64, run via Rosetta 2 — same `as`/`ld -lSystem`
pipeline as [Chapter 5](./05-hello-world-macos.md), just with a computed
exit code standing in for "the arithmetic was correct"):

```asm
movb $-3, %al
movb $-5, %bl
imul %bl                   # signed 1-operand form: al * bl -> ax
movzbl %al, %edi            # (-3) * (-5) = 15

xor %edx, %edx              # clear the high half before dividing — required
mov $47, %eax
mov $5, %ecx
div %ecx                    # 47 / 5 -> eax=9 (quotient), edx=2 (remainder)
add %eax, %edi
add %edx, %edi               # 15 + 9 + 2 = 26
```

```
$ ./muldiv_test ; echo $?
26
```

`imul` also has 2- and 3-operand forms (`imul %ebx, %eax` → `eax *= ebx`,
truncated to 32 bits; `imul $3, %ebx, %eax` → `eax = ebx * 3`) that discard
the high half entirely instead of splitting across two registers — useful
when you know the result fits, and considerably less fiddly to use than
the 1-operand form.

## The full conditional-jump family: unsigned vs. signed

[Chapter 9](./09-control-flow.md) covered `jz`/`jnz`/`jl`/`jg` for the
common case. The reason there's a *second*, differently-named set of
comparison jumps (`ja`/`jb`/`jae`/`jbe`) isn't redundancy — it's that
`cmp` alone can't tell whether you meant the two operands as signed or
unsigned numbers, so **which jump you choose is what decides that**:

| Comparison | Unsigned jump | Signed jump | Flags checked |
|---|---|---|---|
| first > second | `ja` | `jg` | unsigned: `CF=0 AND ZF=0` · signed: `ZF=0 AND SF=OF` |
| first < second | `jb` | `jl` | unsigned: `CF=1` · signed: `SF≠OF` |
| first ≥ second | `jae` | `jge` | unsigned: `CF=0` · signed: `SF=OF` |
| first ≤ second | `jbe` | `jle` | unsigned: `CF=1 OR ZF=1` · signed: `ZF=1 OR SF≠OF` |

The signed column comparing `SF` to `OF` (instead of just reading `SF`
directly) is the part worth sitting with: if there *was* a signed
overflow, the sign bit's plain meaning is wrong — checking it against
whether an overflow happened is what makes signed comparisons correct even
right at the overflow boundary.

Verified — the same bit pattern (`0xFFFFFFFF`, i.e. `-1` as a signed
32-bit value) genuinely means different things to `ja`/`jb` than to
`jg`/`jl`:

```asm
mov $-1, %eax        # 0xFFFFFFFF
mov $1, %ecx
cmp %ecx, %eax

ja  unsigned_above    # TAKEN: 0xFFFFFFFF is huge as an unsigned number
jg  signed_greater     # NOT taken: -1 is less than 1 as a signed number
jl  signed_less         # TAKEN: confirms the signed read is correct
```

```
$ ./jumps_test ; echo $?
9
```
(`9 = 1 (ja taken) + 8 (jl taken)`; the `jb`/`jg` branches correctly did
*not* fire — see [the full test](./20-macros-and-a-vm-project.md#appendix-a-the-four-verification-programs)
if you want the exact accounting.)

**`cmov` — a branch-free alternative:** `cmova`/`cmovb`/`cmovg`/`cmovl`
(and their `ae`/`be`/`ge`/`le` siblings) copy a value *conditionally*,
using the exact same flag logic as the table above, without ever jumping:

```asm
cmp %ecx, %eax
cmova %r9d, %r8d      # r8d := r9d, only if eax was unsigned-above ecx
```

This matters beyond style: a jump the CPU can't predict correctly (data-
dependent branches in a tight loop) stalls the pipeline; `cmov` has no
branch to mispredict at all. It's the same underlying idea `cmp`+`jz`
scaled up — the CPU already computed the flags; `cmov` just reads them
without ever redirecting `rip`.

## Assembler macros — GAS's own system

FASM's macro system (`macro`/`forward`/`reverse`/`match`, from the
gpfault series linked above) is powerful but assembler-specific. GAS — the
assembler this whole book uses — has its own, smaller system that solves
the identical problem: **avoiding hand-written, near-identical repetition**.
Three constructs cover most of what you'll actually reach for:

**`.macro`/`.endm`** — a named, parameterized chunk of assembly, expanded
inline wherever it's invoked. Parameters are referenced with a leading
backslash (`\name`), which is how the assembler tells "this gets
substituted as text" apart from a real instruction operand:

```asm
.macro add_const reg, amount
    add $\amount, \reg
.endm

add_const %edi, 3       # expands to: add $3, %edi
```

**`.rept N` / `.endr`** — an assembly-time loop: the enclosed instructions
are duplicated N times *at build time*, with zero runtime cost (there's no
actual loop instruction in the output — just N copies of the body, one
after another).

**`.if` / `.else` / `.endif`** — conditional assembly: the condition is
evaluated by the assembler itself while it's building the file, not by the
CPU while the program runs. Whichever branch doesn't match is never even
turned into machine code — useful for things like "only emit the
debug-logging instructions if `DEBUG_BUILD == 1`."

All three, verified together in one program:

```asm
.macro add_const reg, amount
    add $\amount, \reg
.endm

add_const %edi, 3
add_const %edi, 4          # edi = 7

.rept 5
    inc %edi
.endr                       # edi = 12

NCONST = 1
.if NCONST == 1
    add $100, %edi            # only this branch is actually assembled
.else
    add $999, %edi
.endif                      # edi = 112
```

```
$ ./macros_test ; echo $?
112
```

## Capstone: a tiny bytecode VM with a jump-table dispatch loop

This is the project "large enough to need" the macros above — a minimal
stack-based virtual machine: a fixed array of opcode bytes (the
"bytecode"), a small memory region acting as the VM's own stack (entirely
separate from the real hardware stack this book covered in
[Chapter 10](./10-functions-and-the-stack.md)), and a loop that reads one
opcode at a time and jumps to the code that implements it. This is the
same fundamental technique a real bytecode interpreter (Python's CPython,
the JVM, Lua) uses at its core — just with four instructions instead of a
few hundred.

**The dispatch trick — a jump table:** instead of a long `cmp`/`je` chain
(check "is it opcode 0? is it opcode 1? ...") the opcode's numeric value
is used directly as an index into a table of code addresses, and one
indirect jump lands exactly where it needs to, regardless of how many
opcodes exist:

```asm
fetch:
    movzbl (%r13,%r12), %eax    # fetch opcode byte (r13 = base of the bytecode array)
    inc %r12                     # advance the VM's own instruction pointer
    jmp *(%r15,%rax,8)             # r15 = base of the table; index by opcode * 8 bytes
```

(`r12`, `r13`, `r14`, `r15` here play the same role [Chapter 6](./06-registers-plain-english.md)
described for general-purpose registers — there's nothing special about
them beyond being convenient callee-saved registers to dedicate to the
VM's own state for the duration of this program.)

**The four opcodes:** `PUSH imm8`, `ADD`, `SUB`, `HALT` (top of stack
becomes the process exit code — so the *real* OS's `exit` syscall doubles
as this toy VM's "return a value" mechanism). `ADD` and `SUB` are close
to identical — pop two bytes, combine them, push the result — which is
exactly the repetition `.macro` exists to collapse into one template:

```asm
.macro binop name, insn
op_\name:
    dec %bl
    movzbl (%r14,%rbx), %eax     # pop b (top of stack)
    dec %bl
    movzbl (%r14,%rbx), %ecx     # pop a (now new top)
    \insn %al, %cl                 # cl = cl <op> al, i.e. a <op> b
    movb %cl, (%r14,%rbx)
    inc %bl
    jmp fetch
.endm

binop add, add     # generates op_add, using the `add` instruction
binop sub, sub      # generates op_sub, using the `sub` instruction
```

Bytecode for `(5 + 3) - 2`, and the verified result:

```asm
prog:
    .byte 0, 5    # PUSH 5
    .byte 0, 3    # PUSH 3
    .byte 1        # ADD    -> stack: [8]
    .byte 0, 2    # PUSH 2
    .byte 2        # SUB    -> stack: [6]
    .byte 3        # HALT   -> exit(6)

jump_table:
    .quad op_push, op_add, op_sub, op_halt
```

```
$ ./vm_test ; echo $?
6
```

**Why the addresses go through `lea label(%rip), %rN` first, instead of
`prog(%r12)` directly:** x86-64's RIP-relative addressing mode — already
covered in [Chapter 8](./08-memory-and-addressing.md) as the position-
independent way to reach a fixed label — doesn't support adding an *index*
register on top of it in the same instruction. The two-step form (load the
base address into a register once with `lea`, then index off *that*
register) works around the limitation and is the standard pattern any time
RIP-relative addressing and array indexing are both needed together.

**What a real bytecode VM adds from here** that this toy one skips for
clarity: a proper growable stack instead of a fixed 64-byte buffer, error
handling for malformed bytecode (this one trusts its input completely),
multi-byte operands, and many more opcodes — but the *shape* (fetch byte →
jump-table dispatch → handler → jump back to fetch) doesn't change as the
instruction set grows. That shape is the actual takeaway of this chapter.

## Appendix A: the four verification programs

Every code block above was extracted from one of four small, complete
programs — assembled with `as -arch x86_64`, linked with `ld -lSystem
-syslibroot $(xcrun --show-sdk-path) -e _main -arch x86_64`, and run via
Rosetta 2 (`arch -x86_64 ./program`), exactly the macOS x86-64 pipeline
[Chapter 5](./05-hello-world-macos.md) established. Each one signals
"correct" by exiting with a specific, hand-computed exit code rather than
printing — a deliberately simple way to verify pure computation (no
syscall write buffer, no format string) when a debugger isn't handy:
`echo $?` right after running is the entire test harness.

| Program | Tests | Expected exit code |
|---|---|---|
| `muldiv_test` | `imul` (signed), `div` (unsigned, with the required `edx` clear) | `26` |
| `jumps_test` | `ja`/`jb`/`jg`/`jl` on the same bit pattern, plus `cmova` | `9` |
| `macros_test` | `.macro`/`.endm`, `.rept`/`.endr`, `.if`/`.else`/`.endif` | `112` |
| `vm_test` | the full capstone VM, running `(5 + 3) - 2` | `6` |
