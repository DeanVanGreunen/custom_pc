; float_demo.asm — basic float arithmetic smoke test
; Computes 3.14 * 2.0 = 6.28, then 6.28 / 3.14 = 2.0, converts back to int.
; Prints results to serial.

        .equ STDOUT, 0x0000

        ; r0 = 3.14, r1 = 2.0
        fldi r0, 3.14
        fldi r1, 2.0

        ; r2 = r0 * r1  →  6.28
        mov  r2, r0
        fmul r2, r1

        ; r3 = r2 / r0  →  2.0
        mov  r3, r2
        fdiv r3, r0

        ; r4 = (int) r3  →  2
        mov  r4, r3
        ftoi r4

        ; print 'r' 'e' 's' '=' digit '\n'
        ldi  r5, 0x72
        out  r5, STDOUT
        ldi  r5, 0x65
        out  r5, STDOUT
        ldi  r5, 0x73
        out  r5, STDOUT
        ldi  r5, 0x3D
        out  r5, STDOUT
        addi r4, 0x30
        out  r4, STDOUT
        ldi  r5, 0x0A
        out  r5, STDOUT

        hlt
