# CPU Vendor & Brand String via CPUID — Linux x86-64 (AT&T syntax)
#
# WHAT: ask the CPU itself, directly, who made it ("GenuineIntel",
# "AuthenticAMD") and its full marketing name ("Intel(R) Core(TM)
# i9-... @ 3.60GHz") - no OS call involved, no file to read. This is
# possibly the single most famous piece of x86 assembly there is: the
# `cpuid` instruction is the reason `/proc/cpuinfo`, every "About This
# Mac"-style panel, and every JIT's feature-detection code can know
# what CPU they're running on at all.
#
# WHY THIS IS x86-64 ONLY - THERE IS NO ARM64 VERSION OF THIS FILE:
# `cpuid` is a real, dedicated x86 instruction, directly callable from
# unprivileged (ring 3 / EL0) code - no syscall needed. ARM64 has
# NOTHING equivalent that user-space code can call directly. The chip-
# identifying registers that exist on ARM64 (MIDR_EL1 and friends) are
# only readable from privileged code (EL1+, kernel mode) - a user-space
# ARM64 program has no direct-instruction path to "what CPU is this" at
# all, and has to ask the OS instead (reading `/proc/cpuinfo` on Linux,
# a `sysctl` call on macOS). This is a real, structural architecture
# difference worth having internalized, not a gap in this collection.
#
# THE MECHANISM: `cpuid` reads whatever value is in `eax` (and
# sometimes `ecx`) as a "leaf" number selecting what information to
# return, then overwrites `eax`/`ebx`/`ecx`/`edx` with the answer.
# Different leaves return completely different kinds of data - this
# file uses three of them:
#
#   - leaf 0: highest supported BASIC leaf in eax; the 12-byte vendor
#     ID string packed across ebx/edx/ecx (that exact order - NOT
#     ebx/ecx/edx, a genuinely easy detail to get backwards).
#   - leaf 0x80000000: highest supported EXTENDED leaf in eax - this
#     MUST be checked before touching any leaf 0x80000001 or above,
#     because those leaves are simply undefined on hardware that
#     doesn't support them. Skipping this check is a real, documented
#     bug class - see 10_cpu_core_count in this same folder for a
#     worked example of exactly what goes wrong without it.
#   - leaves 0x80000002, 0x80000003, 0x80000004: 16 bytes each of the
#     48-byte brand string, packed eax/ebx/ecx/edx per leaf, three
#     leaves back to back.
#
# COMPLEXITY: O(1) - a fixed, small number of `cpuid` calls regardless
# of anything else. O(1) space - two small fixed-size buffers.
#
# BUILD & RUN:
#   gcc linux_x86_64.s -o cpu_info -no-pie && ./cpu_info
#
# VERIFIED OUTPUT (captured on this machine — Docker Desktop's x86-64
# container layer on Apple Silicon, itself worth noticing: the VENDOR
# string claims Intel, but the BRAND string reveals it's actually
# virtualized):
#   vendor: GenuineIntel
#   brand: VirtualApple @ 2.50GHz

.global main
.section .text
main:
    push %rbp
    mov %rsp, %rbp

    # ---- vendor string: leaf 0 ----
    mov $0, %eax
    cpuid
    lea vendor(%rip), %rsi
    mov %ebx, (%rsi)              # bytes 0-3  (NOT ecx - see header)
    mov %edx, 4(%rsi)               # bytes 4-7
    mov %ecx, 8(%rsi)                 # bytes 8-11
    movb $0, 12(%rsi)                   # null terminator

    lea vendor(%rip), %rsi
    lea vendor_fmt(%rip), %rdi
    xor %al, %al
    call printf

    # ---- brand string: leaves 0x80000002-0x80000004 ----
    mov $0x80000000, %eax               # ask: how many extended
    cpuid                                  # leaves does this CPU support?
    cmp $0x80000004, %eax
    jb brand_unsupported                     # fewer than needed -> bail out
                                                # safely instead of reading
                                                # an undefined leaf's garbage

    lea brand(%rip), %rdi
    mov $0x80000002, %eax
    cpuid
    mov %eax, (%rdi)
    mov %ebx, 4(%rdi)
    mov %ecx, 8(%rdi)
    mov %edx, 12(%rdi)

    mov $0x80000003, %eax
    cpuid
    mov %eax, 16(%rdi)
    mov %ebx, 20(%rdi)
    mov %ecx, 24(%rdi)
    mov %edx, 28(%rdi)

    mov $0x80000004, %eax
    cpuid
    mov %eax, 32(%rdi)
    mov %ebx, 36(%rdi)
    mov %ecx, 40(%rdi)
    mov %edx, 44(%rdi)
    movb $0, 48(%rdi)

    lea brand(%rip), %rsi
    lea brand_fmt(%rip), %rdi
    xor %al, %al
    call printf
    jmp done
brand_unsupported:
    lea unsupported_msg(%rip), %rdi
    xor %al, %al
    call printf
done:
    xor %eax, %eax
    pop %rbp
    ret

.section .bss
    .lcomm vendor, 13
    .lcomm brand, 49

.section .data
vendor_fmt:
    .asciz "vendor: %s\n"
brand_fmt:
    .asciz "brand: %s\n"
unsupported_msg:
    .asciz "brand string not supported on this CPU\n"
