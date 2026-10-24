// Towers of Hanoi — macOS ARM64 (Apple Silicon, native)
//
// Same algorithm, same 3-disk puzzle, same expected output as
// linux_x86_64.s in this folder — read that file's header for the full
// "move n-1 out of the way, move the big disk, move n-1 back" mental
// model and why two recursive calls (rather than factorial's one) is
// the actual point of this example.
//
// WHY THIS FILE NEEDS TO SAVE x30 EXPLICITLY, TWICE OVER: `hanoi`
// calls itself (`bl hanoi`) TWICE and also calls `print_move` once, in
// between. Every one of those three calls overwrites x30 with a new
// return address (Chapter 6/10 of the tutorial). Without saving x30 to
// the stack before the FIRST of those three calls and restoring it
// after the LAST, this function would have no way to find its own way
// back to whoever called it - it would instead return to wherever
// print_move (or the second `hanoi` call) itself was told to return.
// This is 06_sieve_of_eratosthenes's print_value bug, at a scale where
// getting it wrong would be far more confusing to debug - three
// candidate places for the return address to get lost instead of one.
//
// BUILD & RUN:
//   as macos_arm64.s -o hanoi.o
//   ld hanoi.o -o hanoi -lSystem \
//      -syslibroot $(xcrun --show-sdk-path) \
//      -e _main -arch arm64 -platform_version macos 11.0 11.0
//   codesign -s - hanoi
//   ./hanoi
//
// VERIFIED OUTPUT: identical 7-move sequence to linux_x86_64.s.

.global _main
.section __TEXT,__text
_main:
    stp x29, x30, [sp, #-16]!
    mov x29, sp

    mov x0, #3      // n = 3 disks
    mov x1, #1        // from peg 1
    mov x2, #3          // to peg 3
    mov x3, #2             // via peg 2
    bl hanoi

    mov x0, #0
    ldp x29, x30, [sp], #16
    ret

// hanoi(n: x0, from: x1, to: x2, via: x3)
//
// All four parameters get saved to fixed stack slots up front, exactly
// like the x86-64 version - the reason is identical: every register
// here is caller-saved, and this function makes three further calls
// total, so anything still needed afterward has to live somewhere a
// call can't touch.
hanoi:
    cbz x0, hanoi_done            // base case: 0 disks -> nothing to do

    sub sp, sp, #48
    stp x29, x30, [sp, #32]         // save OUR OWN return address here -
    str x0, [sp, #24]                 // this is the save the header
    str x1, [sp, #16]                   // comment above is about
    str x2, [sp, #8]
    str x3, [sp, #0]

    // First call: move n-1 disks from `from` to `via`, using `to` as
    // the spare this time - to's and via's ROLES swap for this call.
    sub x0, x0, #1                  // new n = n - 1
    // x1 (from) is already correct, unchanged
    mov x4, x2                        // save `to` before x2/x3 get reused
    mov x2, x3                          // new to  = old via
    mov x3, x4                            // new via = old to
    bl hanoi

    // The largest disk (disk n) moves directly, exactly once - read
    // the ORIGINAL n/from/to straight back off the stack, untouched by
    // whatever the call above did to x0-x3 internally.
    ldr x0, [sp, #24]
    ldr x1, [sp, #16]
    ldr x2, [sp, #8]
    bl print_move

    // Second call: move the same n-1 disks from `via` to `to`, using
    // the now-empty `from` as the spare.
    ldr x0, [sp, #24]
    sub x0, x0, #1                  // new n    = n - 1
    ldr x1, [sp, #0]                  // new from = old via
    ldr x2, [sp, #8]                    // new to   = old to (unchanged)
    ldr x3, [sp, #16]                     // new via  = old from
    bl hanoi

    ldp x29, x30, [sp, #32]           // restore OUR return address
    add sp, sp, #48
hanoi_done:
    ret

// print_move(disk: x0, from: x1, to: x2) - Apple ABI's stack layout
// for THREE variadic args, confirmed against clang's own -S output for
// the equivalent C call: three consecutive 8-byte slots starting at
// [sp, #0], in argument order (disk, from, to) - a direct extension of
// the single-vararg pattern from the tutorial's Chapter 11.
print_move:
    sub sp, sp, #48
    str x30, [sp, #24]     // save OUR return address before calling printf -
                              // parked past the 3 vararg slots, not inside them
    str x0, [sp, #0]          // disk  -> vararg slot 0
    str x1, [sp, #8]            // from  -> vararg slot 1
    str x2, [sp, #16]             // to    -> vararg slot 2
    adrp x0, fmt@PAGE
    add x0, x0, fmt@PAGEOFF
    bl _printf
    ldr x30, [sp, #24]
    add sp, sp, #48
    ret

.section __TEXT,__cstring
fmt:
    .asciz "move disk %ld from %ld to %ld\n"
