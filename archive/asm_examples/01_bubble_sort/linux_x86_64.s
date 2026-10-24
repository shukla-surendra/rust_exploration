# Bubble Sort — Linux x86-64 (AT&T syntax)
#
# WHAT: sort an array of integers ascending, in place.
#
# WHY BUBBLE SORT, AS A FIRST SORTING EXAMPLE: it's the sort whose
# mechanism needs no explanation beyond "compare neighbors, swap if
# they're in the wrong order, repeat" — every other comparison sort
# (selection, insertion, merge, quick) is a variation on deciding WHICH
# pair to compare next, not a fundamentally different idea. Get the
# nested-loop-plus-swap shape solid here and it transfers directly.
#
# THE MENTAL MODEL: one full pass over the array bubbles the single
# LARGEST remaining element all the way to its final position at the
# end — exactly the way the biggest bubble in a glass of soda rises to
# the top fastest. After pass 1, the largest element is guaranteed
# correctly placed at the last index; after pass 2, the second-largest
# is placed at the second-to-last index; and so on. That's the whole
# proof of correctness: each pass shrinks the "not yet guaranteed
# sorted" region by exactly one element from the right.
#
#     [5, 2, 8, 1, 9, 3]
# pass 0: compare (5,2)->swap, (2,8)->ok, (8,1)->swap, (8,9)->ok, (9,3)->swap
#       -> [2, 5, 1, 8, 3, 9]   <- 9 (the max) has bubbled to the end
# pass 1: -> [2, 1, 5, 3, 8]    <- 8 now correctly placed, one before it
# ...continues until the unsorted region shrinks to nothing.
#
# WHY THE INNER LOOP SHRINKS EACH PASS (n-1-i, not always n-1): after i
# completed passes, the LAST i elements are already known-correct — re-
# comparing them again would just waste work re-confirming what pass i
# already proved. This is the one detail that separates "correct but
# needlessly slow" bubble sort from the standard version.
#
# COMPLEXITY: O(n^2) time in the worst and average case — n-1 passes,
# each doing up to n-1 comparisons. O(1) extra space — every swap
# happens in place, directly in the array's own memory.
#
# BUILD & RUN (inside the Linux x86-64 Docker container from the
# asm-zero-to-hero tutorial, Chapter 2):
#   gcc linux_x86_64.s -o bubble_sort -no-pie && ./bubble_sort
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

    mov $0, %r12                 # r12 = i, the outer pass counter
outer_loop:
    cmp $5, %r12                   # 5 = n-1 (n=6): after 5 passes, done
    jge sort_done
    mov $0, %r13                     # r13 = j, this pass's inner index
inner_loop:
    mov $5, %rbx                       # rbx = n-1
    sub %r12, %rbx                       # rbx = (n-1) - i: shrinks each pass
    cmp %rbx, %r13
    jge inner_done
    lea arr(%rip), %rsi                    # rsi = base address of arr
    mov (%rsi,%r13,8), %rax                  # rax = arr[j]
    mov 8(%rsi,%r13,8), %rdx                   # rdx = arr[j+1]  (next quad, +8 bytes)
    cmp %rdx, %rax
    jle no_swap                                  # already in order -> skip
    mov %rdx, (%rsi,%r13,8)                        # arr[j]   = old arr[j+1]
    mov %rax, 8(%rsi,%r13,8)                         # arr[j+1] = old arr[j]
no_swap:
    inc %r13
    jmp inner_loop
inner_done:
    inc %r12
    jmp outer_loop
sort_done:
    call print_array          # show the array after sorting

    xor %eax, %eax
    pop %rbp
    ret

# print_array: walks `arr` and prints all 6 elements space-separated,
# then a newline. Split out as its own function purely so it can be
# called twice (before/after) without duplicating the loop — the same
# "don't repeat yourself" reasoning that applies in any language.
print_array:
    push %rbp
    mov %rsp, %rbp
    push %r14              # r14 must survive the printf call below -
    mov $0, %r14              # it's callee-saved, so this is safe to do
print_loop:
    cmp $6, %r14
    jge print_done
    lea arr(%rip), %rsi
    mov (%rsi,%r14,8), %rsi    # rsi = arr[i]  (reuses rsi: base was only
                                  # needed to compute this, not after)
    lea fmt(%rip), %rdi
    xor %al, %al                   # 0 vector-register args (Chapter 11)
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
    .quad 5, 2, 8, 1, 9, 3
fmt:
    .asciz "%ld "
newline:
    .asciz "\n"
