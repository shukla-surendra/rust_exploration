# Linear Search — Linux x86-64 (AT&T syntax)
#
# WHAT: find the index of a target value in an UNSORTED array, or
# report it isn't present.
#
# WHY THIS ONE IS WORTH WRITING BY HAND, DESPITE BEING "OBVIOUS": it's
# the baseline every other search algorithm in this collection gets
# measured against. Binary search (04_binary_search) exists ONLY
# because it beats this — and "beats this" only means something once
# you've felt what this one costs: an unlucky search (target is last,
# or absent) touches every single element, one at a time, with no way
# to skip ahead. That's the entire justification for binary search's
# extra requirement (the array must be sorted) — it's the price paid to
# escape exactly this worst case.
#
# THE ALGORITHM: walk the array start to end, comparing each element
# against the target. Stop and report the index the instant a match is
# found; if the walk reaches the end with no match, report "not found."
# No cleverness possible without more information about the array (see
# binary search for what changes once you're GUARANTEED it's sorted).
#
# COMPLEXITY: O(n) time worst case (target absent, or at the last
# index) and O(1) average-case-if-you-get-lucky, but there's no
# guarantee of luck — O(n) is the honest bound to quote. O(1) space.
#
# BUILD & RUN:
#   gcc linux_x86_64.s -o linear_search -no-pie && ./linear_search
#
# VERIFIED OUTPUT (searching for 45, present at index 5, then 99, absent):
#   index: 5
#   index: -1

.global main
.section .text
main:
    push %rbp
    mov %rsp, %rbp

    mov $45, %rdi              # search for a value that IS present
    call linear_search
    mov %rax, %rsi
    lea found_fmt(%rip), %rdi
    xor %al, %al
    call printf

    mov $99, %rdi              # search for a value that is NOT present
    call linear_search
    mov %rax, %rsi
    lea found_fmt(%rip), %rdi
    xor %al, %al
    call printf

    xor %eax, %eax
    pop %rbp
    ret

# linear_search(target: rdi) -> index in rax, or -1 if not found.
# -1 (rather than, say, 0) is the conventional "not found" sentinel
# specifically because 0 is itself a valid index - returning 0 for
# "not found" would be indistinguishable from "found at the start."
linear_search:
    lea arr(%rip), %rsi
    mov $0, %rcx                   # rcx = current index
search_loop:
    cmp $8, %rcx                     # walked off the end?
    jge not_found
    mov (%rsi,%rcx,8), %rax            # rax = arr[current index]
    cmp %rdi, %rax
    je found                             # match - stop immediately, don't
    inc %rcx                               # keep scanning past a hit
    jmp search_loop
found:
    mov %rcx, %rax
    ret
not_found:
    mov $-1, %rax
    ret

.section .data
arr:
    .quad 23, 5, 78, 12, 3, 45, 9, 67
found_fmt:
    .asciz "index: %ld\n"
