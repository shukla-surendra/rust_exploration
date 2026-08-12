// Linear Search — macOS ARM64 (Apple Silicon, native)
//
// Same algorithm, same array, same expected output as
// linux_x86_64.s in this folder — read that file's header for the
// full mental model and why this is the baseline binary search
// (04_binary_search) exists to beat.
//
// See 01_bubble_sort/macos_arm64.s for the syntax notes shared across
// this whole collection.
//
// BUILD & RUN:
//   as macos_arm64.s -o linear_search.o
//   ld linear_search.o -o linear_search -lSystem \
//      -syslibroot $(xcrun --show-sdk-path) \
//      -e _main -arch arm64 -platform_version macos 11.0 11.0
//   codesign -s - linear_search
//   ./linear_search
//
// VERIFIED OUTPUT:
//   index: 5
//   index: -1

.global _main
.section __TEXT,__text
_main:
    stp x29, x30, [sp, #-16]!
    mov x29, sp

    mov x0, #45              // search for a value that IS present
    bl linear_search
    bl print_index

    mov x0, #99              // search for a value that is NOT present
    bl linear_search
    bl print_index

    mov x0, #0
    ldp x29, x30, [sp], #16
    ret

// linear_search(target: x0) -> index in x0, or -1 if not found.
linear_search:
    adrp x1, arr@PAGE
    add x1, x1, arr@PAGEOFF
    mov x2, #0                    // x2 = current index
search_loop:
    cmp x2, #8                      // walked off the end?
    b.ge not_found
    ldr x3, [x1, x2, lsl #3]          // x3 = arr[current index]
    cmp x3, x0
    b.eq found                          // match - stop immediately
    add x2, x2, #1
    b search_loop
found:
    mov x0, x2
    ret
not_found:
    mov x0, #-1
    ret

// print_index: prints whatever's in x0 as "index: %ld\n" - split out
// since it's called twice, with the same Apple-ABI variadic-on-stack
// pattern used throughout this collection.
print_index:
    sub sp, sp, #32
    stp x29, x30, [sp, #16]
    add x29, sp, #16
    mov x9, sp
    str x0, [x9]
    adrp x0, found_fmt@PAGE
    add x0, x0, found_fmt@PAGEOFF
    bl _printf
    ldp x29, x30, [sp, #16]
    add sp, sp, #32
    ret

.section __TEXT,__cstring
found_fmt:
    .asciz "index: %ld\n"

.section __DATA,__data
arr:
    .quad 23, 5, 78, 12, 3, 45, 9, 67
