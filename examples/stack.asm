; stack.asm — compute factorial(3) using recursive subroutine calls, then
; display the result digit on the text framebuffer and serial port.
;
; Text framebuffer: base 0xC000, 2 bytes per cell.
;   byte 0: ASCII character
;   byte 1: attribute  (0x07 = light grey on black)

        .equ STDOUT,  0x0000
        .equ FB_BASE, 0xC000
        .equ ATTR,    0x07

        ldi  r0, 3
        call factorial          ; r1 = 3! = 6

        ; display result digit on text FB
        ldi  r5, FB_BASE
        mov  r2, r1
        addi r2, 0x30           ; ASCII digit
        stb  [r5], r2
        addi r5, 1
        ldi  r3, ATTR
        stb  [r5], r3

        ; emit to serial
        out  r2, STDOUT
        ldi  r2, 0x0A
        out  r2, STDOUT

        hlt

; ── factorial(r0) → r1 ────────────────────────────────────────────────────────
; Recursive.  Base case: r0 <= 1 → r1 = 1.

factorial:
        cmpi r0, 1
        jle  base_case

        push r0
        subi r0, 1
        call factorial          ; r1 = (n-1)!
        pop  r0                 ; restore n

        ; r1 = n * r1  via repeated addition
        ldi  r2, 0
        mov  r3, r0
mul_loop:
        add  r2, r1
        subi r3, 1
        jnz  mul_loop
        mov  r1, r2
        ret

base_case:
        ldi  r1, 1
        ret
