# RAM Size via the `sysinfo` Syscall — Linux x86-64 (AT&T syntax)
#
# WHAT: ask the kernel how much RAM this machine has, how much is
# free, and how much swap exists - a genuinely different KIND of
# question from 09/10 in this folder, and that difference is the point.
#
# WHY THIS NEEDS A SYSCALL, NOT AN INSTRUCTION LIKE CPUID: the CPU
# itself has no idea how much RAM is installed - RAM is a separate
# physical component the CPU talks to over a memory bus, and enumerating
# it (reading it from firmware tables at boot, tracking what's
# allocated since) is entirely the KERNEL's job, not the CPU's. There is
# no `raminfo` instruction the way there's a `cpuid` instruction - this
# information only exists inside the kernel's own bookkeeping, so the
# only way to get it is to ask the kernel directly, via a syscall.
#
# THE MECHANISM: `sysinfo(struct sysinfo *info)` (syscall number 99 on
# Linux x86-64) fills a fixed-layout struct with a whole family of
# related numbers in one call - not just RAM. The struct layout below
# was verified by dumping every 8-byte offset from a real call and
# cross-checking each value against `free -b`'s own output on the same
# machine at the same moment - every field lines up exactly.
#
#     offset   field        what it means
#     0        uptime       seconds since boot
#     8-24     loads[3]     1/5/15-minute load averages (raw, fixed-point)
#     32       totalram     total usable RAM  <- this file's target
#     40       freeram      currently free RAM
#     48       sharedram    RAM used by shared memory
#     56       bufferram    RAM used by buffers
#     64       totalswap    total swap space
#     72       freeswap     free swap space
#     80       procs        number of running processes (16-bit field)
#     88       totalhigh    "high memory" - always 0 on a 64-bit kernel,
#     96       freehigh        a legacy 32-bit-kernel concept
#     104      mem_unit     multiply totalram/freeram/etc. by THIS to
#                              get bytes - it's 1 on essentially every
#                              modern 64-bit system, but reading it
#                              instead of assuming 1 is the technically
#                              correct thing to do
#
# COMPLEXITY: O(1) - one syscall, one fixed-size struct, no loop of any
# kind. The entire "complexity" here is knowing the struct layout, not
# any algorithm.
#
# BUILD & RUN:
#   gcc linux_x86_64.s -o ram_size -no-pie && ./ram_size
#
# VERIFIED OUTPUT (this machine's Docker x86-64 container; cross-
# checked byte-for-byte against `free -b` run in the same container at
# the same moment):
#   total RAM: 8321232896 bytes

.global main
.section .text
main:
    push %rbp
    mov %rsp, %rbp
    sub $128, %rsp              # room for the whole struct sysinfo -
                                   # 112 bytes needed, rounded up and
                                   # kept 16-byte aligned for the call

    mov $99, %rax                   # syscall number 99 = sysinfo (Ch.12)
    mov %rsp, %rdi                    # arg 1: pointer to fill
    syscall

    mov 104(%rsp), %rcx                 # mem_unit
    mov 32(%rsp), %rax                    # totalram
    imul %rcx, %rax                         # actual bytes = totalram * mem_unit
    mov %rax, %rsi
    lea fmt(%rip), %rdi
    xor %al, %al
    call printf

    xor %eax, %eax
    add $128, %rsp
    pop %rbp
    ret

.section .data
fmt:
    .asciz "total RAM: %ld bytes\n"
