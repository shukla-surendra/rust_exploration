# Binary Search — Linux x86-64 (AT&T syntax)
#
# WHAT: find the index of a target value in a SORTED array, or report
# it isn't present - in far fewer comparisons than linear search
# (03_linear_search) needs.
#
# WHY SORTED-NESS IS THE WHOLE TRICK: linear search can't skip anything
# because an unsorted array gives no information - checking element 4
# tells you nothing about element 5. A SORTED array changes that
# completely: checking the MIDDLE element doesn't just check one value,
# it tells you which HALF of the remaining array the target could
# possibly be in, and lets you discard the other half entirely,
# unchecked. That's the entire mechanism - one comparison eliminates
# half the remaining search space, every time.
#
# THE ALGORITHM: maintain a `low` and `high` bound on where the target
# could still be. Check the middle element between them: if it equals
# the target, done. If the target is SMALLER, the target (if present)
# must be to the left, so move `high` to just before the middle -
# discarding the entire right half. If the target is LARGER, move `low`
# to just after the middle, discarding the entire left half. Repeat
# until either found, or `low` crosses past `high` (the search space
# has shrunk to nothing - the target genuinely isn't there).
#
#     arr:  [1, 3, 5, 8, 12, 19, 25, 34, 45, 67]   target = 25
#     low=0 high=9  mid=4 -> arr[4]=12 < 25 -> low=5   (discard left half)
#     low=5 high=9  mid=7 -> arr[7]=34 > 25 -> high=6  (discard right half)
#     low=5 high=6  mid=5 -> arr[5]=19 < 25 -> low=6
#     low=6 high=6  mid=6 -> arr[6]=25 == 25 -> FOUND at index 6
#
# Four comparisons found a value in a 10-element array - linear search
# would have needed 7 (it's at index 6, the 7th element checked
# start-to-end). The gap only widens as the array grows: linear search
# scales O(n), binary search scales O(log n) - doubling the array adds
# ONE more comparison to binary search's worst case, not twice as many.
#
# THE ONE THING THAT MUST BE TRUE FOR THIS TO WORK AT ALL: the array
# must already be sorted. Binary search on an unsorted array doesn't
# just risk being slow - it can flatly miss a target that's actually
# present, because discarding a "half" only throws away guaranteed-
# wrong territory when sortedness guarantees which side the target
# would have to be on.
#
# COMPLEXITY: O(log n) time - each comparison halves the remaining
# search space, so the number of comparisons needed is the number of
# times n can be halved before reaching 1. O(1) space (this iterative
# version - a recursive version would pay O(log n) call-stack space
# instead, the same trade-off named throughout this collection).
#
# BUILD & RUN:
#   gcc linux_x86_64.s -o binary_search -no-pie && ./binary_search
#
# VERIFIED OUTPUT (searching for 25, present at index 6, then 6, absent):
#   index: 6
#   index: -1

.global main
.section .text
main:
    push %rbp
    mov %rsp, %rbp

    mov $25, %rdi               # search for a value that IS present
    call binary_search
    mov %rax, %rsi
    lea found_fmt(%rip), %rdi
    xor %al, %al
    call printf

    mov $6, %rdi                # search for a value that is NOT present
    call binary_search
    mov %rax, %rsi
    lea found_fmt(%rip), %rdi
    xor %al, %al
    call printf

    xor %eax, %eax
    pop %rbp
    ret

# binary_search(target: rdi) -> index in rax, or -1 if not found.
binary_search:
    mov $0, %r8               # r8 = low
    mov $9, %r9                 # r9 = high (n-1, n=10)
bs_loop:
    cmp %r9, %r8                  # low > high -> search space is empty
    jg not_found
    mov %r8, %rax                   # mid = (low + high) / 2
    add %r9, %rax
    mov $0, %rdx                      # zero rdx: div reads the full
    mov $2, %rcx                        # 128-bit rdx:rax as the dividend
    div %rcx                              # rax = mid (quotient); rdx (the
                                             # remainder) is unused here
    lea arr(%rip), %rsi
    mov (%rsi,%rax,8), %r10                 # r10 = arr[mid]
    cmp %rdi, %r10
    je found
    jl go_right                               # arr[mid] < target -> look right
    mov %rax, %r9                               # arr[mid] > target: discard the
    dec %r9                                       # right half -> high = mid - 1
    jmp bs_loop
go_right:
    mov %rax, %r8                # discard the left half -> low = mid + 1
    inc %r8
    jmp bs_loop
found:
    ret                        # rax already holds mid from just above
not_found:
    mov $-1, %rax
    ret

.section .data
arr:
    .quad 1, 3, 5, 8, 12, 19, 25, 34, 45, 67
found_fmt:
    .asciz "index: %ld\n"
