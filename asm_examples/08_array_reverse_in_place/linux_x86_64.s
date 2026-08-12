# Array Reverse In-Place — Linux x86-64 (AT&T syntax)
#
# WHAT: reverse the order of an array's elements, using no second
# array - only the array's own memory.
#
# WHY THIS IS WORTH ITS OWN FILE, DESPITE LOOKING TRIVIAL: it's the
# cleanest possible example of the TWO-POINTER technique - one index
# walking in from the start, one walking in from the end, meeting
# somewhere in the middle. That exact shape (not the reversal itself)
# is what recurs constantly: palindrome checks, partitioning a range,
# "does this array read the same both directions" problems all reuse
# this same two-pointer skeleton with a different thing happening at
# each step. Get the loop's STOPPING CONDITION solid here - the one
# detail every off-by-one bug in this family comes from - and it
# transfers directly.
#
# THE MENTAL MODEL: swap the first and last elements, then the second
# and second-to-last, and so on, walking both ends inward one step at a
# time until they meet (odd-length array) or cross (even-length array).
# Each swap is a full three-step exchange (temp = a; a = b; b = temp) -
# the same pattern the tutorial's Chapter 13 string-reversal fragment
# already introduced, applied here to a plain integer array instead of
# a null-terminated string.
#
#     [10, 20, 30, 40, 50, 60]     left=0  right=5
#     swap(0,5): [60,20,30,40,50,10]   left=1  right=4
#     swap(1,4): [60,50,30,40,20,10]   left=2  right=3
#     swap(2,3): [60,50,40,30,20,10]   left=3  right=2  <- left >= right, stop
#
# WHY `left < right` (STRICTLY less than) IS THE CORRECT STOPPING
# CONDITION: for an EVEN-length array (as above), the two pointers
# cross without ever landing on the same index (3, then would-be 2) -
# `left < right` catches that the instant they cross. For an ODD-length
# array, they eventually land on the SAME middle index - swapping an
# element with itself is harmless, but a loop that kept going past that
# point (`left <= right` instead) would swap it right back and undo
# nothing productive; `left < right` correctly stops one step earlier,
# right when the middle element is reached, needing no swap at all.
#
# COMPLEXITY: O(n) time - exactly n/2 swaps, each O(1). O(1) extra
# space - the entire point of doing this "in place" rather than
# building a new reversed array.
#
# BUILD & RUN:
#   gcc linux_x86_64.s -o array_reverse -no-pie && ./array_reverse
#
# VERIFIED OUTPUT:
#   10 20 30 40 50 60
#   60 50 40 30 20 10

.global main
.section .text
main:
    push %rbp
    mov %rsp, %rbp

    call print_array          # show the array before reversing

    mov $0, %r12                 # r12 = left, walks in from the start
    mov $5, %r13                    # r13 = right (n-1, n=6), walks in
reverse_loop:                         # from the end
    cmp %r13, %r12
    jge reverse_done                    # left >= right -> fully reversed
    lea arr(%rip), %rsi
    mov (%rsi,%r12,8), %rax               # rax = arr[left]
    mov (%rsi,%r13,8), %rdx                 # rdx = arr[right]
    mov %rdx, (%rsi,%r12,8)                   # arr[left]  = old arr[right]
    mov %rax, (%rsi,%r13,8)                     # arr[right] = old arr[left]
    inc %r12
    dec %r13
    jmp reverse_loop
reverse_done:
    call print_array          # show the array after reversing

    xor %eax, %eax
    pop %rbp
    ret

print_array:
    push %rbp
    mov %rsp, %rbp
    push %r14
    mov $0, %r14
print_loop:
    cmp $6, %r14
    jge print_done
    lea arr(%rip), %rsi
    mov (%rsi,%r14,8), %rsi
    lea fmt(%rip), %rdi
    xor %al, %al
    call printf
    inc %r14
    jmp print_loop
print_done:
    lea newline(%rip), %rdi
    xor %al, %al
    call printf
    pop %r14
    pop %rbp
    ret

.section .data
arr:
    .quad 10, 20, 30, 40, 50, 60
fmt:
    .asciz "%ld "
newline:
    .asciz "\n"
