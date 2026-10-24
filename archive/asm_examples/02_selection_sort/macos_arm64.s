// Selection Sort — macOS ARM64 (Apple Silicon, native)
//
// Same algorithm, same array, same expected output as linux_x86_64.s
// in this folder — read that file's header for the full mental model
// (find-the-minimum, one swap per pass, why that's a genuinely
// different memory-write pattern from bubble sort's many small swaps).
//
// See 01_bubble_sort/macos_arm64.s for the syntax notes shared by every
// file in this collection (three-operand arithmetic, separate ldr/str,
// the Apple-ABI variadic-printf stack convention) — not repeated here.
//
// BUILD & RUN:
//   as macos_arm64.s -o selection_sort.o
//   ld selection_sort.o -o selection_sort -lSystem \
//      -syslibroot $(xcrun --show-sdk-path) \
//      -e _main -arch arm64 -platform_version macos 11.0 11.0
//   codesign -s - selection_sort
//   ./selection_sort
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

    mov x19, #0                 // x19 = i, index being filled this pass
outer_loop:
    cmp x19, #5                   // i < n-1
    b.ge sort_done
    mov x21, x19                    // x21 = min_idx, starts as i itself
    add x20, x19, #1                  // x20 = j, starts scanning at i+1
inner_loop:
    cmp x20, #6
    b.ge inner_done
    adrp x1, arr@PAGE
    add x1, x1, arr@PAGEOFF
    ldr x2, [x1, x20, lsl #3]               // x2 = arr[j]
    ldr x3, [x1, x21, lsl #3]                 // x3 = arr[min_idx]  (current best)
    cmp x2, x3
    b.ge no_update                              // arr[j] isn't smaller -> keep looking
    mov x21, x20                                  // found a new smallest -> remember its index
no_update:
    add x20, x20, #1
    b inner_loop
inner_done:
    cmp x19, x21                     // did the minimum move at all?
    b.eq no_swap                       // already in place -> skip the swap entirely
    adrp x1, arr@PAGE
    add x1, x1, arr@PAGEOFF
    ldr x2, [x1, x19, lsl #3]
    ldr x3, [x1, x21, lsl #3]
    str x3, [x1, x19, lsl #3]              // arr[i]       = the found minimum
    str x2, [x1, x21, lsl #3]                // arr[min_idx] = what used to be at i
no_swap:
    add x19, x19, #1
    b outer_loop
sort_done:
    bl print_array          // show the array after sorting

    mov x0, #0
    ldp x29, x30, [sp], #16
    ret

print_array:
    sub sp, sp, #48
    stp x29, x30, [sp, #16]
    add x29, sp, #16
    str x22, [sp, #32]
    mov x22, #0
print_loop:
    cmp x22, #6
    b.ge print_done
    adrp x1, arr@PAGE
    add x1, x1, arr@PAGEOFF
    ldr x8, [x1, x22, lsl #3]
    mov x9, sp
    str x8, [x9]
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
