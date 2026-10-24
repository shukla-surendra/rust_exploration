# Towers of Hanoi — Linux x86-64 (AT&T syntax)
#
# WHAT: move a stack of N disks from one peg to another, one disk at a
# time, never placing a larger disk on top of a smaller one, using a
# third "spare" peg - and print every individual move required.
#
# WHY THIS IS THE RIGHT NEXT RECURSION EXAMPLE: the asm-zero-to-hero
# tutorial's recursive factorial (Chapter 10) makes ONE recursive call
# per level. Hanoi makes TWO - and that's the entire
# point of including it here. With one recursive call, saving "whatever
# I still need after the call returns" is straightforward (factorial
# only needs `n` itself). With two calls, there's real WORK to do
# BETWEEN them (printing the current move) using values that must have
# survived the first call untouched, and a second call afterward that
# needs yet another combination of those same values. This is the
# example that actually forces you to think carefully about what state
# a recursive call needs preserved across it, and for how long.
#
# THE INSIGHT (three moves in disguise as one recursive definition):
# moving N disks from `from` to `to` (using `via` as the spare) is
# exactly: (1) move the top N-1 disks from `from` to `via` (using `to`
# as ITS spare this time) - now the largest disk sits alone and
# uncovered; (2) move that one largest disk directly from `from` to
# `to` - one physical move, always legal, since nothing is stacked on
# it; (3) move the N-1 disks now sitting on `via` over to `to` (using
# `from` as the spare this time, since `from` is now empty). Both
# recursive calls are the SAME problem, smaller, with the three pegs'
# ROLES rotated - not a different algorithm, the same one at a smaller
# scale.
#
#     hanoi(3, from=1, to=3, via=2):
#       hanoi(2, from=1, to=2, via=3)   <- move 2 disks out of the way first
#         hanoi(1, from=1, to=3, via=2)
#           move disk 1: 1 -> 3
#         move disk 2: 1 -> 2
#         hanoi(1, from=3, to=2, via=1)
#           move disk 1: 3 -> 2
#       move disk 3: 1 -> 3              <- the largest disk, moved once
#       hanoi(2, from=2, to=3, via=1)   <- bring the 2 disks back over
#         ...
#
# Base case: hanoi(0, ...) does nothing at all - there's no disk to
# move, so the recursion simply returns.
#
# COMPLEXITY: O(2^n - 1) moves - each level of recursion does exactly
# one physical move plus two half-sized recursive calls, so the move
# count doubles (plus one) with every additional disk. This exponential
# blowup is itself a well-known fact worth having ready: 64 disks (the
# legend the puzzle is named for) would take longer than the current
# age of the universe to complete, one move per second, which is
# exactly why "the algorithm is simple" and "the algorithm is fast" are
# two completely independent claims. O(n) call-stack space - one stack
# frame per disk, at the deepest point of the recursion.
#
# BUILD & RUN:
#   gcc linux_x86_64.s -o hanoi -no-pie && ./hanoi
#
# VERIFIED OUTPUT (n=3, from peg 1 to peg 3, via peg 2):
#   move disk 1 from 1 to 3
#   move disk 2 from 1 to 2
#   move disk 1 from 3 to 2
#   move disk 3 from 1 to 3
#   move disk 1 from 2 to 1
#   move disk 2 from 2 to 3
#   move disk 1 from 1 to 3

.global main
.section .text
main:
    push %rbp
    mov %rsp, %rbp

    mov $3, %rdi     # n = 3 disks
    mov $1, %rsi        # from peg 1
    mov $3, %rdx           # to peg 3
    mov $2, %rcx              # via peg 2
    call hanoi

    xor %eax, %eax
    pop %rbp
    ret

# hanoi(n: rdi, from: rsi, to: rdx, via: rcx)
#
# Every register here is caller-saved (Chapter 10's table), and this
# function itself makes TWO further calls - so anything it still needs
# after either call has to be saved somewhere that survives a call,
# which for values this short-lived means the stack, not a register.
# All four parameters get pushed up front, in a fixed, known order, so
# they can be read back by a fixed stack offset at every point below
# that still needs them.
hanoi:
    cmp $0, %rdi
    jle hanoi_done                # base case: 0 disks -> nothing to do

    push %rdi                       # [rsp+24] = n
    push %rsi                         # [rsp+16] = from
    push %rdx                           # [rsp+8]  = to
    push %rcx                             # [rsp+0]  = via

    # First call: move n-1 disks from `from` to `via`, using `to` as
    # the spare this time - to's and via's ROLES swap for this call.
    mov %rdi, %r8
    dec %r8
    mov %rcx, %r9                # save `via` before rdx/rcx get reused
    mov %rdx, %r10                 # save `to` before rdx/rcx get reused
    mov %r8, %rdi                    # new n   = n - 1
    mov %r9, %rdx                      # new to  = old via
    mov %r10, %rcx                       # new via = old to
    call hanoi

    # The largest disk (disk n) moves directly, exactly once - read the
    # ORIGINAL n/from/to straight back off the stack, untouched by
    # whatever the call above did to rdi/rsi/rdx/rcx internally.
    mov 24(%rsp), %rdi
    mov 16(%rsp), %rsi
    mov 8(%rsp), %rdx
    call print_move

    # Second call: move the same n-1 disks from `via` to `to`, using
    # the now-empty `from` as the spare.
    mov 24(%rsp), %rdi
    dec %rdi                        # new n    = n - 1
    mov (%rsp), %rsi                  # new from = old via
    mov 8(%rsp), %rdx                   # new to   = old to (unchanged)
    mov 16(%rsp), %rcx                    # new via  = old from
    call hanoi

    add $32, %rsp              # pop all 4 saved values at once
hanoi_done:
    ret

# print_move(disk: rdi, from: rsi, to: rdx)
print_move:
    push %rbp
    mov %rsp, %rbp
    mov %rdi, %r8              # stash all 3 inputs before rearranging -
    mov %rsi, %r9                 # printf's argument order (rsi, rdx, rcx)
    mov %rdx, %rcx                   # doesn't match this function's own
    mov %r9, %rdx                       # (rdi, rsi, rdx) parameter order
    mov %r8, %rsi
    lea fmt(%rip), %rdi
    xor %al, %al
    call printf
    pop %rbp
    ret

.section .data
fmt:
    .asciz "move disk %ld from %ld to %ld\n"
