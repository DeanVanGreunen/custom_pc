; fibonacci.asm — compute the first 16 Fibonacci numbers and store them
; in memory starting at 0x0200, 2 bytes each (little-endian word).

        .equ RESULT_BASE, 0x0200
        .equ COUNT,       16

        ldi  r0, 0          ; a = 0
        ldi  r1, 1          ; b = 1
        ldi  r2, COUNT      ; loop counter
        ldi  r3, RESULT_BASE

loop:
        st   [r3], r0       ; mem[r3] = a
        addi r3, 2          ; advance by 2 (word size)
        mov  r4, r1         ; tmp = b
        add  r1, r0         ; b = a + b
        mov  r0, r4         ; a = old b
        subi r2, 1
        jnz  loop

        hlt
