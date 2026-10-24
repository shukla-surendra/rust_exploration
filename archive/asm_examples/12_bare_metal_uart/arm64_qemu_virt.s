.section .text
.global _start
_start:
    adrp x0, stack_top
    add x0, x0, :lo12:stack_top
    mov sp, x0

    bl uart_init

    adrp x0, msg
    add x0, x0, :lo12:msg
    bl uart_print

hang:
    wfe
    b hang

// UART_BASE: QEMU's "virt" machine memory-maps a PL011 UART here -
// this exact address is a property of QEMU's virt machine model, the
// ARM64 equivalent of x86's port 0x3F8 being "just where COM1 lives."
.equ UART_BASE, 0x09000000
.equ UART_DR,   0x00   // Data Register - write a byte here to transmit
.equ UART_FR,   0x18   // Flag Register - bit 5 = TXFF (transmit FIFO full)
.equ UART_IBRD, 0x24   // Integer Baud Rate Divisor
.equ UART_FBRD, 0x28   // Fractional Baud Rate Divisor
.equ UART_LCRH, 0x2C   // Line Control Register
.equ UART_CR,   0x30   // Control Register

// uart_init - the real PL011 spec, and the actual bug this file
// hit during testing: skip this and UARTFR.TXFF reads back as
// permanently SET (the UART reports "transmit FIFO full" forever,
// even though nothing was ever sent), so uart_putc's wait loop below
// spins forever and NOTHING prints - confirmed by instruction-tracing
// the hang with `qemu-system-aarch64 -d in_asm`, which showed
// execution stuck at the `tst`/`b.ne` poll, never reaching the store.
// Initializing the control register - the ARM64/MMIO equivalent of
// the x86 file's LCR/FCR/MCR writes - is not optional ceremony.
uart_init:
    mov x2, #UART_BASE
    mov w3, #0
    str w3, [x2, #UART_CR]           // disable the UART while configuring it

    mov w3, #13
    str w3, [x2, #UART_IBRD]           // baud-rate divisor (integer part)
    mov w3, #1
    str w3, [x2, #UART_FBRD]             // baud-rate divisor (fractional part)

    mov w3, #0x70                          // 8 data bits (WLEN=11) + FIFO enable
    str w3, [x2, #UART_LCRH]

    mov w3, #0x301                           // UARTEN | TXE | RXE
    str w3, [x2, #UART_CR]
    ret

// uart_print(x0 = pointer to a null-terminated string)
uart_print:
    ldrb w1, [x0]
    cbz w1, uart_print_done
    bl uart_putc
    add x0, x0, #1
    b uart_print
uart_print_done:
    ret

// uart_putc(w1 = character) - same hardware handshake as the x86 file
// in this folder, just through a memory LOAD instead of a port IN:
// poll the Flag Register until the transmit FIFO isn't full, THEN
// store the byte to the Data Register.
uart_putc:
    stp x0, x30, [sp, #-16]!
    mov x2, #UART_BASE
uart_wait:
    ldr w3, [x2, #UART_FR]
    tst w3, #(1 << 5)          // TXFF bit
    b.ne uart_wait
    strb w1, [x2, #UART_DR]
    ldp x0, x30, [sp], #16
    ret

.section .data
msg:
    .asciz "Hello from bare metal, via real MMIO!\n"

.section .bss
.align 4
.skip 4096
stack_top:
