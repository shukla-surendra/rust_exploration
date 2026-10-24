// Bubble Sort — macOS ARM64 (Apple Silicon, native)
//
// Same algorithm, same array, same expected output as linux_x86_64.s
// in this folder — read that file's header first for the full mental
// model (the "biggest bubble rises to the top each pass" proof and why
// the inner loop shrinks by one each pass). This file exists to make
// the SYNTAX contrast between the two architectures concrete on a real,
// non-trivial program, not just a hello-world.
//
// WHAT'S GENUINELY DIFFERENT FROM THE x86-64 VERSION, AND WHY:
//   - Three-operand arithmetic (`sub x21, x21, x19` computes x21 = x21
//     - x19, leaving both sources untouched) versus x86-64's two-
//     operand form that overwrites one of its own inputs.
//   - `ldr`/`str` are always separate from arithmetic - x86-64's
//     `mov (%rsi,%r13,8), %rax` fuses "compute this address" and "read
//     from it" into one instruction; ARM64 never does that.
//   - Calling printf here needs the Apple-ABI variadic-argument-on-the-
//     -stack pattern from the asm-zero-to-hero tutorial's Chapter 11 -
//     genuinely the most surprising line in this file if you haven't
//     hit it before. Standard AAPCS64 (Linux ARM64) would just pass the
//     value in a register; Apple's ABI specifically does not, for
//     variadic calls.
//
// BUILD & RUN (native, no Docker needed on Apple Silicon):
//   as macos_arm64.s -o bubble_sort.o
//   ld bubble_sort.o -o bubble_sort -lSystem \
//      -syslibroot $(xcrun --show-sdk-path) \
//      -e _main -arch arm64 -platform_version macos 11.0 11.0
//   codesign -s - bubble_sort
//   ./bubble_sort
//
// VERIFIED OUTPUT:
//   5 2 8 1 9 3
//   1 2 3 5 8 9

.global _main
.section __TEXT,__text
_main:
    stp x29, x30, [sp, #-16]!
    mov x29, sp

    bl print_array          // show the array before sorting

    mov x19, #0                 // x19 = i, the outer pass counter
outer_loop:
    cmp x19, #5                   // 5 = n-1 (n=6): after 5 passes, done
    b.ge sort_done
    mov x20, #0                     // x20 = j, this pass's inner index
inner_loop:
    mov x21, #5                       // x21 = n-1
    sub x21, x21, x19                   // x21 = (n-1) - i: shrinks each pass
    cmp x20, x21
    b.ge inner_done
    adrp x1, arr@PAGE                     // x1 = base address of arr
    add x1, x1, arr@PAGEOFF
    ldr x2, [x1, x20, lsl #3]               // x2 = arr[j]   (lsl #3 = *8)
    add x3, x20, #1
    ldr x4, [x1, x3, lsl #3]                  // x4 = arr[j+1]
    cmp x2, x4
    b.le no_swap                                // already in order -> skip
    str x4, [x1, x20, lsl #3]                     // arr[j]   = old arr[j+1]
    str x2, [x1, x3, lsl #3]                        // arr[j+1] = old arr[j]
no_swap:
    add x20, x20, #1
    b inner_loop
inner_done:
    add x19, x19, #1
    b outer_loop
sort_done:
    bl print_array          // show the array after sorting

    mov x0, #0
    ldp x29, x30, [sp], #16
    ret

// print_array: walks `arr` and prints all 6 elements space-separated,
// then a newline. x22 is callee-saved (AAPCS64: x19-x28), so it safely
// survives the repeated `bl _printf` calls inside the loop.
print_array:
    sub sp, sp, #48                // 16 for our own x29/x30 + x22, plus
    stp x29, x30, [sp, #16]           // 16 more as the vararg stack slot
    add x29, sp, #16                    // printf reads from, per call
    str x22, [sp, #32]
    mov x22, #0
print_loop:
    cmp x22, #6
    b.ge print_done
    adrp x1, arr@PAGE
    add x1, x1, arr@PAGEOFF
    ldr x8, [x1, x22, lsl #3]               // x8 = arr[i]
    mov x9, sp
    str x8, [x9]              // Apple ABI: variadic args go on the
                                  // stack, not in a register - see
                                  // Chapter 11 of the tutorial for the
                                  // clang -S trace that pinned this down
    adrp x0, fmt@PAGE
    add x0, x0, fmt@PAGEOFF
    bl _printf
    add x22, x22, #1
    b print_loop
print_done:
    adrp x0, newline@PAGE
    add x0, x0, newline@PAGEOFF
    bl _printf
    ldr x22, [sp, #32]
    ldp x29, x30, [sp, #16]
    add sp, sp, #48
    ret

.section __TEXT,__cstring
fmt:
    .asciz "%ld "
newline:
    .asciz "\n"

.section __DATA,__data
arr:
    .quad 5, 2, 8, 1, 9, 3
