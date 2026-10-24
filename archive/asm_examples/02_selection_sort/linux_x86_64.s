# Selection Sort — Linux x86-64 (AT&T syntax)
#
# WHAT: sort an array of integers ascending, in place.
#
# WHY THIS ONE RIGHT AFTER BUBBLE SORT: same O(n^2) complexity, same
# "shrinking unsorted region" idea as 01_bubble_sort - but it inverts
# WHERE the work happens, which is the actual thing worth noticing.
# Bubble sort does a swap on nearly every comparison, writing to memory
# constantly as elements slowly migrate. Selection sort does exactly
# ONE swap per pass, no matter how large the array is - it spends the
# whole pass just SEARCHING for the right element, then commits with a
# single write. Same asymptotic time complexity, radically different
# write behavior - worth having ready as an answer to "which sort
# writes to memory less."
#
# THE MENTAL MODEL: on pass i, scan the entire unsorted region (from
# index i to the end) to find the index of its SMALLEST element, then
# swap that smallest element into position i. After the swap, index i
# is permanently correct, and the unsorted region shrinks by one from
# the left (mirror image of bubble sort, which shrinks from the right).
#
#     [5, 2, 8, 1, 9, 3]
# pass i=0: smallest in [5,2,8,1,9,3] is 1 at index 3 -> swap(0,3)
#        -> [1, 2, 8, 5, 9, 3]
# pass i=1: smallest in [2,8,5,9,3] (from index 1) is 2, already there
#        -> [1, 2, 8, 5, 9, 3]   (no-op swap - see the `no_swap` guard)
# pass i=2: smallest in [8,5,9,3] is 3 at index 5 -> swap(2,5)
#        -> [1, 2, 3, 5, 9, 8]
# ...continues until the unsorted region shrinks to nothing.
#
# WHY THE `min_idx == i` CHECK MATTERS: without it, every pass still
# performs a swap even when the smallest element is ALREADY at index i
# - functionally harmless (swapping a value with itself), but it's
# free correctness to skip, and the guard is exactly what makes "one
# swap per pass, or zero" a true statement instead of "always one."
#
# COMPLEXITY: O(n^2) time (same nested-scan shape as bubble sort - n-1
# passes, each scanning up to n-1 remaining elements to find a
# minimum), O(1) extra space, and at most n-1 total swaps - versus
# bubble sort's potentially much larger number of swaps on a badly-
# ordered array, which is the concrete trade-off this file opened with.
#
# BUILD & RUN:
#   gcc linux_x86_64.s -o selection_sort -no-pie && ./selection_sort
#
# VERIFIED OUTPUT:
#   5 2 8 1 9 3
#   1 2 3 5 8 9

.global main
.section .text
main:
    push %rbp
    mov %rsp, %rbp

    call print_array          # show the array before sorting

    mov $0, %r12                 # r12 = i, index being filled this pass
outer_loop:
    cmp $5, %r12                   # i < n-1 (the last element needs no
    jge sort_done                    # pass of its own - it's whatever's left)
    mov %r12, %r14                     # r14 = min_idx, starts as i itself
    mov %r12, %r13                       # r13 = j, starts scanning at i+1
    inc %r13
inner_loop:
    cmp $6, %r13
    jge inner_done
    lea arr(%rip), %rsi
    mov (%rsi,%r13,8), %rax                # rax = arr[j]
    mov (%rsi,%r14,8), %rdx                  # rdx = arr[min_idx]  (current best)
    cmp %rdx, %rax
    jge no_update                              # arr[j] isn't smaller -> keep looking
    mov %r13, %r14                                # found a new smallest -> remember its index
no_update:
    inc %r13
    jmp inner_loop
inner_done:
    cmp %r12, %r14                     # did the minimum move at all?
    je no_swap                           # already in place -> skip the swap entirely
    lea arr(%rip), %rsi
    mov (%rsi,%r12,8), %rax
    mov (%rsi,%r14,8), %rdx
    mov %rdx, (%rsi,%r12,8)                # arr[i]       = the found minimum
    mov %rax, (%rsi,%r14,8)                  # arr[min_idx] = what used to be at i
no_swap:
    inc %r12
    jmp outer_loop
sort_done:
    call print_array          # show the array after sorting

    xor %eax, %eax
    pop %rbp
    ret

print_array:
    push %rbp
    mov %rsp, %rbp
    push %r15
    mov $0, %r15
print_loop:
    cmp $6, %r15
    jge print_done
    lea arr(%rip), %rsi
    mov (%rsi,%r15,8), %rsi
    lea fmt(%rip), %rdi
    xor %al, %al
    call printf
    inc %r15
    jmp print_loop
print_done:
    lea newline(%rip), %rdi
    xor %al, %al
    call printf
    pop %r15
    pop %rbp
    ret

.section .data
arr:
    .quad 5, 2, 8, 1, 9, 3
fmt:
    .asciz "%ld "
newline:
    .asciz "\n"
