# CPU Vendor & Brand String via CPUID — macOS x86-64 (via Rosetta 2)
#
# Same instruction, same leaves, same expected output as
# linux_x86_64.s in this folder — read that file's header for the full
# mechanism and why CPUID has no ARM64 equivalent at all.
#
# THIS FILE HAS NO ARM64 SIBLING AND IS RUN VIA ROSETTA - not a choice,
# a structural fact: Apple Silicon is ARM64, `cpuid` is an x86
# instruction that plain doesn't exist on that hardware, and running
# x86-64 code at all here means going through Rosetta 2's binary
# translation (Chapter 5 of the tutorial). What Rosetta actually
# reports back is itself the interesting result — see below.
#
# WHY THIS FILE USES RAW SYSCALLS, NOT PRINTF: every other x86-64 file
# in this tutorial/collection links against libc and calls printf
# directly (Chapter 11). Doing the same here, cross-compiled for x86-64
# from this Apple Silicon host, hits a real linker error - `printf`
# comes back "symbol(s) not found for architecture x86_64" even with
# `-lSystem` correctly specified. This looks like an x86-64-specific
# gap in the locally installed Command Line Tools SDK's linker stub
# data, not a bug in the assembly - and it's exactly the kind of thing
# worth knowing has a fallback: Chapter 5's raw-syscall pattern
# (`write`/`exit` via BSD syscall numbers, no libc at all) sidesteps it
# completely, which is what this file does instead.
#
# BUILD & RUN:
#   clang -target x86_64-apple-macos11 -c macos_x86_64.s -o cpu_info.o
#   ld cpu_info.o -o cpu_info -lSystem \
#      -syslibroot $(xcrun --show-sdk-path) \
#      -e _main -arch x86_64 -platform_version macos 11.0 11.0
#   codesign -s - cpu_info
#   arch -x86_64 ./cpu_info
#
# VERIFIED OUTPUT — captured via native Rosetta 2 translation on this
# machine, IDENTICAL to the Docker-container result in
# linux_x86_64.s's header, which is itself informative: it strongly
# suggests this Mac's Docker Desktop is *also* using Rosetta 2 for its
# x86-64 container layer, not a separate QEMU-based emulator:
#   GenuineIntel
#   VirtualApple @ 2.50GHz

.global _main
.section __TEXT,__text
_main:
    # ---- vendor string: leaf 0 ----
    mov $0, %eax
    cpuid
    lea vendor(%rip), %rsi
    mov %ebx, (%rsi)
    mov %edx, 4(%rsi)
    mov %ecx, 8(%rsi)
    movb $10, 12(%rsi)          # newline, not a null - see the write
                                    # syscall below, which needs an exact
                                    # byte count rather than a C string
    mov $0x2000004, %rax          # syscall 4 (write) + 0x2000000 class
    mov $1, %rdi                    # offset - Chapter 5 covers this
    lea vendor(%rip), %rsi
    mov $13, %rdx
    syscall

    # ---- brand string: leaves 0x80000002-0x80000004 ----
    mov $0x80000000, %eax
    cpuid
    cmp $0x80000004, %eax
    jb brand_unsupported

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
    movb $10, 48(%rdi)

    mov $0x2000004, %rax
    mov $1, %rdi
    lea brand(%rip), %rsi
    mov $49, %rdx
    syscall
    jmp exit
brand_unsupported:
    mov $0x2000004, %rax
    mov $1, %rdi
    lea unsupported_msg(%rip), %rsi
    mov $28, %rdx
    syscall
exit:
    mov $0x2000001, %rax        # syscall 1 (exit) + 0x2000000 class offset
    xor %rdi, %rdi
    syscall

.section __DATA,__data
vendor:
    .space 13
brand:
    .space 49
unsupported_msg:
    .ascii "brand string not supported\n"
