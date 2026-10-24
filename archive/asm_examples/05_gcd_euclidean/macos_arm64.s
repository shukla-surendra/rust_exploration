// GCD — Euclidean Algorithm — macOS ARM64 (Apple Silicon, native)
//
// Same algorithm, same numbers, same expected output as
// linux_x86_64.s in this folder — read that file's header for the full
// "why gcd(a,b) == gcd(b, a mod b)" derivation and the Euclidean
// algorithm's O(log(min(a,b))) step bound.
//
// THE GENUINELY DIFFERENT PART: computing `a mod b`. x86-64's `div`
// hands you the remainder for free in `rdx` as a side effect of doing
// the division. ARM64 has no instruction that produces a remainder
// directly at all - `udiv` only gives you the quotient. Getting the
// remainder needs a second, explicit step: `msub` (multiply-subtract)
// computes `dst = minuend - (a * b)` in one instruction, so
// `remainder = a - (a/b)*b` becomes exactly one `udiv` followed by one
// `msub`. This is a real, structural RISC-vs-CISC difference, not a
// stylistic one: x86-64's `div` does more work per instruction; ARM64
// spells the same computation out as two separate, explicit steps.
//
// BUILD & RUN:
//   as macos_arm64.s -o gcd.o
//   ld gcd.o -o gcd -lSystem \
//      -syslibroot $(xcrun --show-sdk-path) \
//      -e _main -arch arm64 -platform_version macos 11.0 11.0
//   codesign -s - gcd
//   ./gcd
//
// VERIFIED OUTPUT:
//   gcd: 6

.global _main
.section __TEXT,__text
_main:
    stp x29, x30, [sp, #-32]!
    mov x29, sp

    mov x0, #48
    mov x1, #18
    bl gcd

    mov x9, sp
    str x0, [x9]
    adrp x0, fmt@PAGE
    add x0, x0, fmt@PAGEOFF
    bl _printf

    mov x0, #0
    ldp x29, x30, [sp], #32
    ret

// gcd(a: x0, b: x1) -> x0
gcd:
gcd_loop:
    cmp x1, #0
    b.eq gcd_done                // b == 0 -> a (still in x0) is the answer
    udiv x2, x0, x1                 // x2 = a / b  (quotient only)
    msub x3, x2, x1, x0               // x3 = a - (x2 * b) = a mod b
    mov x0, x1                          // new a = old b
    mov x1, x3                            // new b = a mod b
    b gcd_loop
gcd_done:
    ret

.section __TEXT,__cstring
fmt:
    .asciz "gcd: %ld\n"
