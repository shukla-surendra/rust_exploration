// Array Reverse In-Place — macOS ARM64 (Apple Silicon, native)
//
// Same algorithm, same array, same expected output as linux_x86_64.s
// in this folder — read that file's header for the full two-pointer
// mental model and why `left < right` (not `<=`) is the correct
// stopping condition for both even- and odd-length arrays.
//
// See 01_bubble_sort/macos_arm64.s for the syntax notes shared across
// this whole collection.
//
// BUILD & RUN:
//   as macos_arm64.s -o array_reverse.o
//   ld array_reverse.o -o array_reverse -lSystem \
//      -syslibroot $(xcrun --show-sdk-path) \
//      -e _main -arch arm64 -platform_version macos 11.0 11.0
//   codesign -s - array_reverse
//   ./array_reverse
//
// VERIFIED OUTPUT:
//   10 20 30 40 50 60
//   60 50 40 30 20 10

.global _main
.section __TEXT,__text
_main:
    stp x29, x30, [sp, #-16]!
    mov x29, sp

    bl print_array          // show the array before reversing

    mov x19, #0                 // x19 = left, walks in from the start
    mov x20, #5                   // x20 = right (n-1, n=6), walks in
reverse_loop:                       // from the end
    cmp x19, x20
    b.ge reverse_done                 // left >= right -> fully reversed
    adrp x1, arr@PAGE
    add x1, x1, arr@PAGEOFF
    ldr x2, [x1, x19, lsl #3]           // x2 = arr[left]
    ldr x3, [x1, x20, lsl #3]             // x3 = arr[right]
    str x3, [x1, x19, lsl #3]               // arr[left]  = old arr[right]
    str x2, [x1, x20, lsl #3]                 // arr[right] = old arr[left]
    add x19, x19, #1
    sub x20, x20, #1
    b reverse_loop
reverse_done:
    bl print_array          // show the array after reversing

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
    .quad 10, 20, 30, 40, 50, 60
