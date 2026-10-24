# Sieve of Eratosthenes — Linux x86-64 (AT&T syntax)
#
# WHAT: find every prime number up to a limit (30, here) - in one pass
# of ELIMINATION rather than checking each number individually.
#
# WHY THE OBVIOUS APPROACH IS AWKWARD: "for each number n from 2 to 30,
# test whether anything from 2 to sqrt(n) divides it" works, but it
# re-derives primality from scratch for every single number - O(n *
# sqrt(n)) total work, and it never reuses what testing n=2 already
# proved about every multiple of 2. The sieve inverts the whole
# approach: instead of asking "is this number prime?" one at a time, it
# starts by assuming everything is prime, then systematically CROSSES
# OUT every multiple of each prime it finds - a number that's still
# standing after all the crossing-out never got proven composite by
# anything smaller, which is exactly what "prime" means.
#
# THE ALGORITHM: keep a boolean array, one entry per number 0..N-1,
# all starting at "not yet marked composite." Walk p from 2 upward: if
# p is still unmarked, it's prime (nothing smaller crossed it out) -
# print it, then cross out every multiple of p starting from p*p
# (anything smaller, like 2p or 3p, was already crossed out by a
# SMALLER prime factor on an earlier pass - p*p is the first multiple
# of p that couldn't have been reached any other way yet).
#
#     numbers 2..29, all start unmarked ("prime")
#     p=2: unmarked -> PRIME. cross out 4,6,8,10,...,28 (every multiple of 2)
#     p=3: unmarked -> PRIME. cross out 9,12,15,...,27 (starting at 3*3=9;
#          6 was already crossed out by p=2)
#     p=4: marked (crossed out by p=2) -> skip, not prime
#     p=5: unmarked -> PRIME. cross out 25 (5*5; 10,15,20 already crossed)
#     p=6: marked -> skip
#     ...stop once p*p exceeds the limit - any composite number <= 29
#          with NO factor <= sqrt(29) can't exist (it would need two
#          factors both > sqrt(29), whose product would exceed 29).
#     result: 2 3 5 7 11 13 17 19 23 29
#
# COMPLEXITY: O(n log log n) time - a genuinely famous, surprising
# bound (each prime p contributes roughly n/p crossing-out steps, and
# summing 1/p over all primes up to n converges to log log n, not log n
# or n itself). O(n) space for the boolean array - the classic
# time-for-space trade this algorithm makes on purpose.
#
# BUILD & RUN:
#   gcc linux_x86_64.s -o sieve -no-pie && ./sieve
#
# VERIFIED OUTPUT:
#   2 3 5 7 11 13 17 19 23 29

.global main
.section .text
main:
    push %rbp
    mov %rsp, %rbp

    # sieve[0..29] lives in .bss, which the OS zero-initializes for us -
    # 0 means "still assumed prime," 1 means "proven composite."
    mov $2, %r12                # r12 = p, the candidate prime
sieve_outer:
    mov %r12, %rax
    imul %r12, %rax               # rax = p*p
    cmp $30, %rax
    jg sieve_done                   # p*p past the limit -> every prime found
    lea sieve(%rip), %rsi
    movb (%rsi,%r12,1), %al
    cmp $0, %al
    jne skip_p                        # already crossed out -> not prime, skip
    mov %r12, %rax                      # start crossing out at p*p, not 2p -
    imul %r12, %rax                       # smaller multiples were already
mark_loop:                                  # crossed out by a smaller prime
    cmp $30, %rax
    jge skip_p
    movb $1, (%rsi,%rax,1)               # cross out this multiple of p
    add %r12, %rax
    jmp mark_loop
skip_p:
    inc %r12
    jmp sieve_outer
sieve_done:
    mov $2, %r13                # walk the finished sieve and print survivors
print_loop:
    cmp $30, %r13
    jge print_done
    lea sieve(%rip), %rsi
    movb (%rsi,%r13,1), %al
    cmp $0, %al
    jne not_prime                     # marked composite -> don't print it
    mov %r13, %rsi
    lea fmt(%rip), %rdi
    xor %al, %al
    call printf
not_prime:
    inc %r13
    jmp print_loop
print_done:
    lea newline(%rip), %rdi
    xor %al, %al
    call printf

    xor %eax, %eax
    pop %rbp
    ret

.section .bss
sieve:
    .skip 30              # 30 zero-initialized bytes: one flag per number

.section .data
fmt:
    .asciz "%ld "
newline:
    .asciz "\n"
