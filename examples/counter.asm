; counter.asm — count from 0 to 9 and emit each digit via serial port 0x00.

        .equ STDOUT, 0x0000
        .equ ZERO,   0x30   ; ASCII '0'

        ldi  r0, 0          ; counter
        ldi  r1, 10         ; limit

loop:
        mov  r2, r0
        addi r2, ZERO       ; r2 = '0' + counter
        out  r2, STDOUT
        addi r0, 1
        cmp  r0, r1
        jlt  loop

        ldi  r2, 0x0A       ; newline
        out  r2, STDOUT
        hlt
