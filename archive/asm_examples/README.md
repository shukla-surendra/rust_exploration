# Famous Algorithms in Assembly

Standalone, runnable example programs — the companion to
[`docs/src/asm-zero-to-hero/`](../docs/src/asm-zero-to-hero/00-overview.md)'s
tutorial. That series teaches the *language*; this folder is worked
practice on classic algorithms once you know it, each one a complete,
independently-readable `.s` file with an inline "why," not just "what."

Every program targets two platforms — **Linux x86-64** (AT&T syntax,
build in the Docker container from the tutorial's Chapter 2) and
**macOS ARM64** (native, Apple Silicon) — and every single file listed
below was actually assembled, linked, and run before being written down;
none of this is "should work."

## Algorithms

| # | Algorithm | What it teaches beyond the algorithm itself |
|---|---|---|
| [01](01_bubble_sort/) | Bubble Sort | The baseline nested-loop-plus-swap shape every comparison sort varies on |
| [02](02_selection_sort/) | Selection Sort | Same O(n²), radically fewer writes — one swap per pass, not many |
| [03](03_linear_search/) | Linear Search | The O(n) baseline binary search exists specifically to beat |
| [04](04_binary_search/) | Binary Search | Why sortedness lets one comparison discard half the search space |
| [05](05_gcd_euclidean/) | GCD (Euclidean Algorithm) | x86-64's `div` gives a remainder for free; ARM64 needs `udiv`+`msub` |
| [06](06_sieve_of_eratosthenes/) | Sieve of Eratosthenes | Elimination instead of per-number testing; O(n log log n) |
| [07](07_towers_of_hanoi/) | Towers of Hanoi | Recursion with TWO calls per level — real state to preserve between them |
| [08](08_array_reverse_in_place/) | Array Reverse In-Place | The two-pointer skeleton (palindrome checks, partitioning, etc. all reuse this) |
| [09](09_cpu_vendor_and_brand/) | CPU Vendor & Brand String (CPUID) | x86-only user-space instruction — ARM64 has no equivalent at all |
| [10](10_cpu_core_count/) | CPU Core Count (CPUID) | A real bug fix: a missing leaf-support check, and why the "fixed" leaf is still unreliable |
| [11](11_ram_size/) | RAM Size (`sysinfo` syscall) | Why hardware questions the CPU itself can't answer need a syscall, not an instruction |

**09–11 are a different category from 01–08**: hardware-introspection
programs, not classic algorithms. 09/10 review a 32-bit NASM program a
reader submitted (asking for CPU core count via `cpuid`) and fix a real
bug in it — a missing check that the CPUID leaf being used is even
supported, without which the result is silently undefined. That bug
reproduced live during testing: the same leaf reports **1** core inside
this machine's Docker x86-64 container, **25** under native Rosetta
2 translation, and the real hardware (an Apple M4 Pro, checked via
`sysctl -n hw.ncpu`) has **12** — three different wrong-or-right answers
across three environments, all captured and documented rather than
described secondhand.

09 and 10 have **no `macos_arm64.s`** and instead ship a
`macos_x86_64.s` (run via Rosetta 2) — `cpuid` is a real x86 instruction
with no ARM64 equivalent at all; a user-space ARM64 program has no
direct-instruction path to CPU identity and has to ask the OS instead
(`/proc/cpuinfo` on Linux, `sysctl` on macOS). 11 is Linux-only for now
(x86-64 + ARM64, via the portable `sysinfo` syscall) — a macOS RAM-size
example needs `sysctl`, a genuinely different mechanism, and isn't built
yet.

| [12](12_bare_metal_uart/) | Bare-Metal UART (port I/O + MMIO) | Real hardware communication — boots directly in QEMU, no OS at all |

**12 is a third, further category**: not an algorithm, not asking an OS
a question — talking to a hardware device *directly*, the way a device
driver actually does. This needs to run bare-metal (no OS underneath)
because `in`/`out` port I/O is blocked outright in ring 3 under any real
OS — there's no way to demonstrate it as an ordinary program the way
everything else in this folder is. Full explanation, plus a real bug
this example hit and how it was diagnosed via instruction tracing, is in
the companion tutorial chapter:
[`docs/src/asm/11-io-ports-and-mmio.md`](../docs/src/asm/11-io-ports-and-mmio.md).

Each folder contains `linux_x86_64.s` and `macos_arm64.s` — same
algorithm, same test data, same expected output, so the two files are
meant to be read side by side. Reading them in numeric order roughly
tracks increasing difficulty; 07 (Hanoi) is deliberately the hardest,
since it's the one that forces you to think carefully about what state
a recursive function has to preserve across *multiple* calls, not just
one.

## Not included here — already fully worked in the tutorial

Factorial (recursive) and Fibonacci (iterative) aren't repeated in this
folder — they're already built, explained, and verified on **three**
platforms (Linux x86-64, Linux ARM64, macOS ARM64) in the tutorial's own
[Chapter 10](../docs/src/asm-zero-to-hero/10-functions-and-the-stack.md)
and
[Chapter 17](../docs/src/asm-zero-to-hero/17-mini-projects.md)
respectively. No point duplicating finished work — go there for those
two.

## Build & run

**Linux x86-64** (inside the `asm-amd64` Docker container — see the
tutorial's [Chapter 2](../docs/src/asm-zero-to-hero/02-toolchain-setup.md)
for how to set it up):

```bash
docker exec asm-amd64 bash -c "cd /work/01_bubble_sort && gcc linux_x86_64.s -o bubble_sort -no-pie && ./bubble_sort"
```

**macOS ARM64** (native, no Docker needed):

```bash
cd 01_bubble_sort
as macos_arm64.s -o bubble_sort.o
ld bubble_sort.o -o bubble_sort -lSystem \
   -syslibroot $(xcrun --show-sdk-path) \
   -e _main -arch arm64 -platform_version macos 11.0 11.0
codesign -s - bubble_sort
./bubble_sort
```

Every folder's own file header has the exact command for that specific
program, plus its verified output to check your own run against.

## A recurring bug worth knowing about before you hit it yourself

Three of these files (06, 07, 08's helper functions) call `printf`
*from inside a function that itself was reached via `bl`* — meaning that
function's own return address, sitting in `x30`, is about to be
overwritten by the `bl _printf` call unless it's explicitly saved first.
06's `print_value` and 07's `print_move` both had this exact bug during
development — caught by testing, not by inspection — and both are now
commented at the point of the fix as a worked example of the mistake,
not just the correct code. If you write a new helper function for macOS
ARM64 that calls another function, and it starts returning to the wrong
place or looping forever, this is the first thing to check.
