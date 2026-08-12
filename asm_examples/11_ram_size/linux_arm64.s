// RAM Size via the `sysinfo` Syscall — Linux ARM64
//
// Same struct, same field offsets, same expected output as
// linux_x86_64.s in this folder — read that file's header for the
// full struct layout table and why this needs a syscall at all rather
// than a direct instruction the way CPU identity does on x86 (see
// 09_cpu_vendor_and_brand for that contrast).
//
// THE ONLY GENUINE DIFFERENCE: the syscall NUMBER. `sysinfo` is 99 on
// Linux x86-64 and 179 on Linux ARM64 - confirmed directly against
// this container's own kernel headers
// (`grep __NR_sysinfo /usr/include/asm-generic/unistd.h`), not assumed.
// The struct `sysinfo` fills in is defined once, in portable C, inside
// the kernel - so its layout (and therefore every offset below) is
// IDENTICAL across architectures; only the number you put in x8 to
// reach it changes, exactly the same lesson Chapter 12 already drew
// from `write`/`exit`.
//
// BUILD & RUN:
//   as linux_arm64.s -o ram_size.o && ld ram_size.o -o ram_size && ./ram_size
//   (or, to link against libc for printf: gcc linux_arm64.s -o ram_size)
//
// VERIFIED OUTPUT (this machine's Docker ARM64 container; matches
// `free -b` in the same container):
//   total RAM: 8321232896 bytes

.global main
.section .text
main:
    stp x29, x30, [sp, #-144]!    // 128 bytes for struct sysinfo, plus
    mov x29, sp                     // our own saved frame - kept 16-
                                       // byte aligned throughout

    mov x8, #179                        // syscall number 179 = sysinfo
    add x0, sp, #16                       // arg 1: pointer to fill,
    svc #0                                   // just past our saved x29/x30

    ldr x2, [sp, #16 + 104]              // mem_unit
    ldr x1, [sp, #16 + 32]                 // totalram
    mul x1, x1, x2                           // actual bytes = totalram * mem_unit

    adrp x0, fmt
    add x0, x0, :lo12:fmt
    bl printf

    mov x0, #0
    ldp x29, x30, [sp], #144
    ret

.section .data
fmt:
    .asciz "total RAM: %ld bytes\n"
