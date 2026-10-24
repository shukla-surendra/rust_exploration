// Binary Search — macOS ARM64 (Apple Silicon, native)
//
// Same algorithm, same array, same expected output as
// linux_x86_64.s in this folder — read that file's header for the
// full "why sortedness lets you discard half the array per comparison"
// mental model and the O(log n) vs. O(n) contrast with linear search.
//
// ONE GENUINELY DIFFERENT CHOICE FROM THE x86-64 VERSION: computing
// `mid = (low + high) / 2`. The x86-64 file uses a real `div`
// instruction (and the rdx-zeroing ritual that comes with it). Here,
// `low` and `high` are always non-negative small integers, so dividing
// by 2 is exactly what a right SHIFT does - `lsr #1` (logical shift
// right by 1 bit) - cheaper than an actual division instruction and
// the idiomatic way any compiler would do this exact division too.
// Worth noticing: this optimization is legal ONLY because the divisor
// is a power of two and the value is non-negative; it is not a general
// substitute for division.
//
// See 01_bubble_sort/macos_arm64.s for the other syntax notes shared
// across this whole collection.
//
// BUILD & RUN:
//   as macos_arm64.s -o binary_search.o
//   ld binary_search.o -o binary_search -lSystem \
//      -syslibroot $(xcrun --show-sdk-path) \
//      -e _main -arch arm64 -platform_version macos 11.0 11.0
//   codesign -s - binary_search
//   ./binary_search
//
// VERIFIED OUTPUT:
//   index: 6
//   index: -1

.global _main
.section __TEXT,__text
_main:
    stp x29, x30, [sp, #-16]!
    mov x29, sp

    mov x0, #25               // search for a value that IS present
    bl binary_search
    bl print_index

    mov x0, #6                // search for a value that is NOT present
    bl binary_search
    bl print_index

    mov x0, #0
    ldp x29, x30, [sp], #16
    ret

// binary_search(target: x0) -> index in x0, or -1 if not found.
binary_search:
    mov x1, #0                  // x1 = low
    mov x2, #9                    // x2 = high (n-1, n=10)
    mov x9, x0                      // save target - x0 is about to be
                                       // reused for the array pointer
bs_loop:
    cmp x1, x2                        // low > high -> search space empty
    b.gt not_found
    add x3, x1, x2                      // mid = (low + high) >> 1
    lsr x3, x3, #1
    adrp x0, arr@PAGE
    add x0, x0, arr@PAGEOFF
    ldr x4, [x0, x3, lsl #3]              // x4 = arr[mid]
    cmp x4, x9
    b.eq found
    b.lt go_right                           // arr[mid] < target -> look right
    sub x2, x3, #1                            // arr[mid] > target: discard the
    b bs_loop                                   // right half -> high = mid - 1
go_right:
    add x1, x3, #1                // discard the left half -> low = mid + 1
    b bs_loop
found:
    mov x0, x3                  // x3 still holds mid from just above
    ret
not_found:
    mov x0, #-1
    ret

// print_index: prints x0 as "index: %ld\n" - Apple-ABI variadic-on-
// stack pattern, same as 03_linear_search/macos_arm64.s.
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
    .quad 1, 3, 5, 8, 12, 19, 25, 34, 45, 67
