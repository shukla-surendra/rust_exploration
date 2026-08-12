# GCD — Euclidean Algorithm — Linux x86-64 (AT&T syntax)
#
# WHAT: the greatest common divisor of two positive integers - the
# largest number that divides both with no remainder.
#
# WHY THE NAIVE APPROACH IS AWKWARD: "try every number from
# min(a,b) down to 1, return the first that divides both evenly" works,
# but it's O(min(a,b)) - for two large numbers with a small GCD, that's
# a huge number of wasted trial divisions. The Euclidean algorithm (one
# of the oldest algorithms in continuous use, literally from Euclid's
# "Elements," circa 300 BCE) finds the answer in a small, bounded
# number of steps regardless of how large a and b are.
#
# THE INSIGHT: gcd(a, b) == gcd(b, a mod b). Any number that divides
# both a and b ALSO divides (a mod b) - because a mod b is just "a,
# minus some whole number of copies of b," and if a common divisor
# evenly divides both a and b, it evenly divides that difference too.
# So the pair (a, b) and the pair (b, a mod b) always share the exact
# same set of common divisors, including the greatest one. Repeating
# this replacement shrinks the numbers fast (b always gets smaller,
# strictly, every step) until one of them hits 0 - at which point the
# OTHER one is the answer, because gcd(x, 0) = x by definition (x is
# trivially the greatest thing that divides both x and 0).
#
#     gcd(48, 18):
#       (48, 18) -> 48 mod 18 = 12 -> (18, 12)
#       (18, 12) -> 18 mod 12 = 6  -> (12, 6)
#       (12, 6)  -> 12 mod 6  = 0  -> (6, 0)
#       b is 0 -> answer is a = 6
#
# COMPLEXITY: O(log(min(a, b))) steps - provably fast; the worst case
# (fewest steps saved per round) happens on consecutive Fibonacci
# numbers, which is where the algorithm's logarithmic bound actually
# comes from historically. O(1) space.
#
# BUILD & RUN:
#   gcc linux_x86_64.s -o gcd -no-pie && ./gcd
#
# VERIFIED OUTPUT:
#   gcd: 6

.global main
.section .text
main:
    push %rbp
    mov %rsp, %rbp

    mov $48, %rdi
    mov $18, %rsi
    call gcd
    mov %rax, %rsi
    lea fmt(%rip), %rdi
    xor %al, %al
    call printf

    xor %eax, %eax
    pop %rbp
    ret

# gcd(a: rdi, b: rsi) -> rax
gcd:
gcd_loop:
    cmp $0, %rsi
    je gcd_done                 # b == 0 -> a (still in rdi) is the answer
    mov %rdi, %rax
    mov $0, %rdx                  # zero rdx before div - see 04_binary_search
    div %rsi                        # rax = a/b (discarded), rdx = a mod b
    mov %rsi, %rdi                    # new a = old b
    mov %rdx, %rsi                      # new b = a mod b
    jmp gcd_loop
gcd_done:
    mov %rdi, %rax
    ret

.section .data
fmt:
    .asciz "gcd: %ld\n"
