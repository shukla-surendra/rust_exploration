// Sieve of Eratosthenes — macOS ARM64 (Apple Silicon, native)
//
// Same algorithm, same limit (30), same expected output as
// linux_x86_64.s in this folder — read that file's header for the full
// "cross out multiples instead of testing each number individually"
// mental model and the O(n log log n) complexity derivation.
//
// See 01_bubble_sort/macos_arm64.s for the syntax notes shared across
// this whole collection (three-operand arithmetic, separate ldr/str,
// the Apple-ABI variadic-printf stack convention).
//
// BUILD & RUN:
//   as macos_arm64.s -o sieve.o
//   ld sieve.o -o sieve -lSystem \
//      -syslibroot $(xcrun --show-sdk-path) \
//      -e _main -arch arm64 -platform_version macos 11.0 11.0
//   codesign -s - sieve
//   ./sieve
//
// VERIFIED OUTPUT:
//   2 3 5 7 11 13 17 19 23 29

.global _main
.section __TEXT,__text
_main:
    stp x29, x30, [sp, #-16]!
    mov x29, sp

    // sieve[0..29] lives in __DATA,__bss (zero-initialized by the OS) -
    // 0 means "still assumed prime," 1 means "proven composite."
    mov x19, #2                 // x19 = p, the candidate prime
sieve_outer:
    mul x1, x19, x19               // x1 = p*p
    cmp x1, #30
    b.gt sieve_done                  // p*p past the limit -> every prime found
    adrp x2, sieve@PAGE
    add x2, x2, sieve@PAGEOFF
    ldrb w3, [x2, x19]
    cbnz w3, skip_p                    // already crossed out -> skip
                                          // x1 already holds p*p from above -
mark_loop:                                 // start crossing out there, since
    cmp x1, #30                              // smaller multiples were already
    b.ge skip_p                                // crossed out by a smaller prime
    mov w4, #1
    strb w4, [x2, x1]                          // cross out this multiple of p
    add x1, x1, x19
    b mark_loop
skip_p:
    add x19, x19, #1
    b sieve_outer
sieve_done:
    mov x20, #2                 // walk the finished sieve, print survivors
print_loop:
    cmp x20, #30
    b.ge print_done
    adrp x2, sieve@PAGE
    add x2, x2, sieve@PAGEOFF
    ldrb w3, [x2, x20]
    cbnz w3, not_prime                 // marked composite -> don't print it
    bl print_value
not_prime:
    add x20, x20, #1
    b print_loop
print_done:
    adrp x0, newline@PAGE
    add x0, x0, newline@PAGEOFF
    bl _printf

    mov x0, #0
    ldp x29, x30, [sp], #16
    ret

// print_value: prints x20 as "%ld " - split out so the print_loop
// above doesn't need its own Apple-ABI stack-vararg boilerplate
// inline. Unlike print_array in 01_bubble_sort, this function calls
// _printf itself - which means it MUST save x30 (its own return
// address) before that call, or _printf's own `bl`-triggered overwrite
// of x30 would clobber the address print_value needs to `ret` to.
// This is exactly the non-leaf-function bug the tutorial's Chapter 10
// warns about, reproduced here on purpose: x19/x20 are callee-saved
// and survive the call for free, but x30 does NOT save itself.
print_value:
    sub sp, sp, #32
    str x30, [sp, #16]
    mov x9, sp
    str x20, [x9]
    adrp x0, fmt@PAGE
    add x0, x0, fmt@PAGEOFF
    bl _printf
    ldr x30, [sp, #16]
    add sp, sp, #32
    ret

.section __TEXT,__cstring
fmt:
    .asciz "%ld "
newline:
    .asciz "\n"

.section __DATA,__bss
sieve:
    .skip 30              // 30 zero-initialized bytes: one flag per number
