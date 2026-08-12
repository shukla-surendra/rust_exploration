# CPU Core Count via CPUID — macOS x86-64 (via Rosetta 2)
#
# Same fix, same leaf, same reasoning as linux_x86_64.s in this folder
# - read that file's header for why the leaf-0x80000000 support check
# is the actual bug fix here, and why leaf 0x80000008 is known-
# unreliable in virtualized/modern environments even once "supported."
#
# No printf here, for the same SDK-linking reason documented in
# 09_cpu_vendor_and_brand/macos_x86_64.s - raw syscalls instead. Since
# there's no printf/`%d` to lean on, this file also keeps the ORIGINAL
# submitted program's hand-rolled decimal-to-ASCII conversion loop
# (build digits right-to-left by repeated division by 10) rather than
# replacing it - that part of the original was already correct, and
# it's a genuinely useful pattern to see working end to end.
#
# BUILD & RUN:
#   clang -target x86_64-apple-macos11 -c macos_x86_64.s -o cpu_cores.o
#   ld cpu_cores.o -o cpu_cores -lSystem \
#      -syslibroot $(xcrun --show-sdk-path) \
#      -e _main -arch x86_64 -platform_version macos 11.0 11.0
#   codesign -s - cpu_cores
#   arch -x86_64 ./cpu_cores
#
# VERIFIED OUTPUT (native Rosetta 2 on this machine - a real Apple M4
# Pro, confirmed via `sysctl -n hw.ncpu` to have 12 real cores):
#   cores (leaf 0x80000008): 25
#
# 25 is not 12. Neither is linux_x86_64.s's own captured result of 1,
# from the SAME leaf run inside this same machine's Docker container.
# Two different virtualization/translation layers, two different WRONG
# answers, neither matching the real hardware - this is exactly the
# unreliability the header above warns about, not a hypothetical.

.global _main
.section __TEXT,__text
_main:
    mov $0x80000000, %eax
    cpuid
    cmp $0x80000008, %eax
    jb not_supported

    mov $0x80000008, %eax
    cpuid
    mov %ecx, %eax
    and $0xff, %eax
    inc %eax

    lea prefix(%rip), %rsi
    mov $0x2000004, %rax
    mov $1, %rdi
    mov $25, %rdx
    syscall

    # decimal-to-ASCII: same reverse-digit technique as the original
    # submitted program, ported to this file's buffer/register layout.
    lea number(%rip), %rdi
    add $10, %rdi
convert:
    xor %edx, %edx
    mov $10, %ebx
    div %ebx                 # eax = eax/10, edx = eax mod 10

    add $'0', %dl
    dec %rdi
    mov %dl, (%rdi)

    test %eax, %eax
    jnz convert

    lea number(%rip), %rsi
    add $10, %rsi
    sub %rdi, %rsi                # rsi = digit count (end - digit_start),
                                     # computed BEFORE rdi is repurposed below
    mov %rsi, %rdx                    # rdx = that digit count
    mov %rdi, %rsi                      # rsi = pointer to the first digit
    mov $0x2000004, %rax
    mov $1, %rdi                          # NOW safe to overwrite rdi with fd
    syscall

    mov $0x2000004, %rax
    mov $1, %rdi
    lea newline(%rip), %rsi
    mov $1, %rdx
    syscall
    jmp exit
not_supported:
    mov $0x2000004, %rax
    mov $1, %rdi
    lea unsupported(%rip), %rsi
    mov $42, %rdx
    syscall
exit:
    mov $0x2000001, %rax
    xor %rdi, %rdi
    syscall

.section __DATA,__data
prefix:
    .ascii "cores (leaf 0x80000008): "
number:
    .space 10
newline:
    .ascii "\n"
unsupported:
    .ascii "leaf 0x80000008 not supported on this CPU\n"
