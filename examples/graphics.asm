; graphics.asm — fill the text-mode frame buffer (base 0xC000) with a
; checkerboard pattern of '*' and ' ' characters using attr 0x07 (white on black).

        .equ FB_BASE,  0xC000
        .equ FB_COLS,  80
        .equ FB_ROWS,  25
        .equ FB_SIZE,  4000    ; 80 * 25 * 2

        .equ CHAR_STAR,  0x2A  ; '*'
        .equ CHAR_SPACE, 0x20  ; ' '
        .equ ATTR,       0x07  ; white fg, black bg

        ldi  r0, FB_BASE       ; r0 = write pointer
        ldi  r1, FB_SIZE       ; r1 = remaining bytes
        ldi  r2, 0             ; r2 = cell index (for checkerboard)

fill_loop:
        ; Compute column = (r2 / 2) % 2 XOR row parity — simplified:
        ; even index → '*', odd → ' '
        mov  r3, r2
        andi r3, 1
        jnz  odd_cell
        ldi  r4, CHAR_STAR
        jmp  write_char
odd_cell:
        ldi  r4, CHAR_SPACE
write_char:
        stb  [r0], r4          ; write character
        addi r0, 1
        ldi  r4, ATTR
        stb  [r0], r4          ; write attribute
        addi r0, 1

        addi r2, 1
        subi r1, 2
        jnz  fill_loop

        hlt
