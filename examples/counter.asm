; counter.asm — count from 0 to 9, writing each digit to the text-mode
; display framebuffer and to serial port 0x00.
;
; Text framebuffer: base 0xC000, 160 cols × 50 rows, 2 bytes per cell:
;   byte 0: ASCII character
;   byte 1: attribute  (bits 3–0 = fg colour, bits 6–4 = bg colour)
;
; Digits are placed on row 0, columns 0–9.

        .equ STDOUT,  0x0000
        .equ FB_BASE, 0xC000
        .equ ATTR,    0x07      ; white-on-black
        .equ ZERO,    0x30      ; ASCII '0'

        ldi  r0, 0              ; r0 = counter  (0 … 9)
        ldi  r1, 10             ; r1 = limit
        ldi  r5, FB_BASE        ; r5 = text-FB write pointer

loop:
        mov  r2, r0
        addi r2, ZERO           ; r2 = ASCII digit

        ; ── write to text display ──────────────────────────────────────
        stb  [r5], r2           ; cell char byte
        addi r5, 1
        ldi  r3, ATTR
        stb  [r5], r3           ; cell attr byte
        addi r5, 1

        ; ── write to serial ───────────────────────────────────────────
        out  r2, STDOUT

        addi r0, 1
        cmp  r0, r1
        jlt  loop

        ldi  r2, 0x0A           ; newline to serial
        out  r2, STDOUT
        hlt
