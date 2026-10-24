# 1. What Is a CPU, Really?

## The kitchen analogy

Imagine a cook who can only do a handful of things: pick up an ingredient
from a labeled spot, put an ingredient down at a labeled spot, add two
ingredients together, compare two ingredients, and jump to a different
step in the recipe if a condition is true. That's it. That cook, working
through a recipe one line at a time, at a blistering pace, **is a CPU**.

- The labeled spots the cook keeps in their own two hands (only a
  handful — maybe 16 or 32) are **registers**: the CPU's own tiny, instant
  storage.
- The pantry full of labeled shelves, much bigger but a short walk away,
  is **RAM (memory)**.
- The recipe itself, written as a strict numbered list of one-instruction
  steps, is your **program** — sitting in memory too, just like the
  ingredients.
- The cook reading the current step, understanding what it means, and
  doing it, over and over, is the **fetch-decode-execute cycle** — the
  entire thing a CPU does, all day, billions of times a second.

Every program you've ever run, in any language, eventually gets reduced to
a recipe this dumb. That's not an insult to the CPU — it's the whole
trick. Simple operations, done unbelievably fast and in unbelievable
volume, add up to everything from a web browser to a video game.

## Fetch, decode, execute — the only loop there is

1. **Fetch**: read the next instruction from memory, at the address the
   **program counter** register points to (called `rip` on x86-64, `pc` on
   ARM64 — a register just like any other, just one the CPU updates for
   you after almost every instruction).
2. **Decode**: figure out what the instruction actually means — "this is
   an add," "this is a jump," etc. — and which registers/memory it needs.
3. **Execute**: actually do it — add the numbers, write the result
   somewhere, jump to a different instruction, whatever it calls for.
4. Move the program counter to the next instruction (or, for a jump, to
   wherever the jump points), and go back to step 1.

That's it. There is no step 5. Everything a computer does — a whole
operating system, a video call, this very sentence being rendered on your
screen — is this four-step loop running so fast that its output looks
instantaneous and its individual steps become invisible.

## Machine code, assembly, and "real" languages — three views of the same thing

They are not three different things stacked on top of each other so much
as **three different ways of writing down the exact same instructions**,
each one a step further from what the CPU actually reads:

**Machine code** is what the CPU actually fetches — pure numbers. An `add`
instruction on x86-64 might literally be the bytes `48 01 D8` sitting in
memory. The CPU doesn't understand words; it understands bit patterns it
was physically built to recognize.

**Assembly language** is machine code with the numbers replaced by names a
human can read — `add rax, rbx` instead of `48 01 D8`. Critically, this
translation is **mechanical and reversible**: one line of assembly becomes
exactly one (sometimes two or three) machine instructions, in a fixed,
predictable way. The tool that does this translation is called an
**assembler** (you'll use one called `as` — literally short for
"assembler" — starting in the next chapter).

**A "real" language** — Rust, C, Python, whatever — is a much higher-level
description of *what you want*, which a **compiler** translates down into
assembly (and from there, the assembler turns it into machine code). This
step is *not* mechanical and reversible the way assembly→machine-code is —
the compiler makes real decisions: which variables live in which
registers, whether to unroll a loop, whether to skip work it can prove is
unnecessary. One line of Rust might become zero instructions (optimized
away entirely) or fifty.

```
Rust/C source  →  [compiler]  →  Assembly  →  [assembler]  →  Machine code  →  CPU
 (your intent)      (makes real          (1:1, readable)      (1:1, pure numbers)
                      decisions)
```

This is exactly why this section exists: assembly is the layer where you
can see precisely what the CPU is going to do, with nothing hidden and
nothing decided for you by a compiler's optimizer.

## Why assembly *looks* so different between x86-64 and ARM64

Short version, expanded a lot more starting in
[Chapter 6](./06-registers-plain-english.md): x86-64 and ARM64 are two
different **instruction set architectures** (ISAs) — two different,
mutually incompatible vocabularies of what "add," "jump," "load from
memory" even mean as bit patterns, how many registers exist and what
they're named, and what an instruction is even allowed to do in one step.

The single biggest structural difference to have in mind going forward:

- **x86-64 is CISC** (Complex Instruction Set Computer) — instructions can
  be short or long (1 to 15 bytes), and a single instruction is allowed to
  reach into memory *and* do arithmetic in the same step (e.g. `add
  rax, [rbx]` adds the value **stored at the address in `rbx`** directly to
  `rax`).
- **ARM64 is RISC** (Reduced Instruction Set Computer) — every instruction
  is exactly 4 bytes, and arithmetic instructions are only ever allowed to
  work on registers, never memory directly. Getting a value from memory
  into a register is *always* its own separate `ldr` (load) instruction
  first.

Neither is "better" in the abstract — it's a genuine, decades-old design
trade-off (fewer, simpler instructions that are easy to pipeline and keep
power-efficient, vs. fewer *lines of assembly* per task at the cost of
instruction complexity) that still shapes real hardware today: it's a
direct reason Apple Silicon (ARM64) gets the battery life it does compared
to x86-64 laptops of similar performance.

## Why the *same* architecture looks different again on Linux vs macOS

This one surprises people the most, and it's not about the CPU at all —
it's about the **operating system**. The chip executes the same
instructions either way; what differs is:

- **The file format the OS expects an executable to be packaged in** —
  Linux wants **ELF**, macOS wants **Mach-O**. Different headers,
  different section-naming conventions (`.text`/`.data` on Linux vs.
  `__TEXT,__text`/`__DATA,__data` on macOS), incompatible with each other.
- **How you ask the OS to do something** (print text, exit the program,
  read a file) — this is a **syscall**, and both the *mechanism* and the
  *numbering* differ between Linux and macOS, even on identical CPU
  hardware. [Chapter 12](./12-syscalls-deep-dive.md) covers this in full;
  you'll already see it firsthand in the very next few chapters' four
  different hello-world programs.

So "x86-64 assembly" is really shorthand for "x86-64 instructions,
packaged and interfaced with the OS a particular way" — and that packaging
is a second, independent axis of difference on top of the CPU architecture
itself. Keeping these two axes (CPU ISA vs. OS conventions) mentally
separate is what makes the four-way split in this section click instead
of feeling like four unrelated things to memorize.

## What's next

[Chapter 2](./02-toolchain-setup.md) gets the actual tools installed —
assembler, linker, debugger — on both macOS and Linux, plus the Docker
trick that lets a single Mac test all four platform combinations. Then
[Chapters 3 through 5](./03-hello-world-linux-x86-64.md) write and *run* a
real program on every one of them.
