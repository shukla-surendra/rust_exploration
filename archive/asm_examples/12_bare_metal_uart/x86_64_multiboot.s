.set MB_MAGIC, 0x1BADB002
.set MB_FLAGS, 0x0
.set MB_CHECKSUM, -(MB_MAGIC + MB_FLAGS)

.section .multiboot
.align 4
.long MB_MAGIC
.long MB_FLAGS
.long MB_CHECKSUM

.section .bss
.align 16
stack_bottom:
.skip 16384
stack_top:

.section .text
.global _start
_start:
    mov $stack_top, %esp

    call uart_init
    lea msg, %esi
    call uart_print

hang:
    hlt
    jmp hang

# --- UART (16550, COM1 @ 0x3F8) init: this is the real init sequence
# every 16550-compatible UART expects, per its hardware spec ---
uart_init:
    mov $0x3F8, %dx
    add $1, %dx
    mov $0x00, %al          # IER: disable all interrupts
    out %al, %dx

    mov $0x3F8, %dx
    add $3, %dx
    mov $0x80, %al          # LCR: set DLAB=1 to expose the baud-rate
    out %al, %dx               # divisor registers at offsets 0/1

    mov $0x3F8, %dx             # DLL: divisor low byte
    mov $0x03, %al                # 115200 / 3 = 38400 baud
    out %al, %dx

    mov $0x3F8, %dx
    add $1, %dx
    mov $0x00, %al           # DLM: divisor high byte
    out %al, %dx

    mov $0x3F8, %dx
    add $3, %dx
    mov $0x03, %al              # LCR: DLAB=0, 8 data bits, no parity,
    out %al, %dx                    # 1 stop bit ("8N1")

    mov $0x3F8, %dx
    add $2, %dx
    mov $0xC7, %al             # FCR: enable + clear FIFOs, 14-byte trigger
    out %al, %dx

    mov $0x3F8, %dx
    add $4, %dx
    mov $0x0B, %al              # MCR: RTS/DSR set, enable IRQs (not used
    out %al, %dx                    # here, but expected by real hardware)
    ret

# uart_print(esi = pointer to a null-terminated string)
uart_print:
    mov (%esi), %al
    test %al, %al
    jz uart_print_done
    call uart_putc
    inc %esi
    jmp uart_print
uart_print_done:
    ret

# uart_putc(al = character) - the actual per-byte hardware handshake:
# poll the Line Status Register's THRE bit until the transmit holding
# register is empty, THEN write the byte - writing before THRE is set
# would silently corrupt or drop the byte on real hardware.
uart_putc:
    push %eax
uart_wait:
    mov $0x3F8, %dx
    add $5, %dx          # LSR: Line Status Register
    in %dx, %al
    test $0x20, %al          # bit 5 = THRE (Transmit Holding Register Empty)
    jz uart_wait
    pop %eax
    mov $0x3F8, %dx
    out %al, %dx
    ret

.section .data
msg:
    .asciz "Hello from bare metal, via real port I/O!\n"
