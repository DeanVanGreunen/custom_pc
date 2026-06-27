; fibonacci.asm — compute the first 16 Fibonacci numbers and display each as
; a 4-digit hex string on the text framebuffer and serial port.
;
; Text framebuffer: base 0xC000, 2 bytes per cell.
;   byte 0: ASCII character
;   byte 1: attribute  (0x0F = white on black)
;
; Register conventions:
;   r0 = current fib value (a)
;   r1 = next fib value (b)
;   r2 = iteration counter
;   r5 = text-FB write pointer  (shared with subroutines — they advance it)
;   r6 = nibble / character temp  (subroutine arg/scratch)
;   r7 = shift amount temp  (subroutine scratch)
;   r3 = attribute byte  (subroutine scratch)

        .equ STDOUT,  0x0000
        .equ FB_BASE, 0xC000
        .equ ATTR,    0x0F

        ldi  r5, FB_BASE
        ldi  r0, 0              ; a = 0
        ldi  r1, 1              ; b = 1
        ldi  r2, 16             ; count

fib_loop:
        cmpi r2, 0
        jz   done

        ; print r0 as 4 hex digits, advancing r5
        call print_hex4

        ; write a space separator
        ldi  r6, 0x20
        stb  [r5], r6
        addi r5, 1
        ldi  r3, ATTR
        stb  [r5], r3
        addi r5, 1
        out  r6, STDOUT

        ; advance fibonacci: a, b = b, a+b
        mov  r4, r1
        add  r1, r0
        mov  r0, r4

        subi r2, 1
        jmp  fib_loop

done:
        hlt

; ── print_hex4 ────────────────────────────────────────────────────────────────
; Print r0 as a 4-digit hex string to FB at [r5], advancing r5 by 8 bytes.
; Also emits each character to serial.
; Clobbers: r3, r6, r7.  Preserves: r0, r1, r2, r4.

print_hex4:
        mov  r6, r0
        ldi  r7, 12
        shr  r6, r7
        andi r6, 0x0F
        call print_nibble

        mov  r6, r0
        ldi  r7, 8
        shr  r6, r7
        andi r6, 0x0F
        call print_nibble

        mov  r6, r0
        ldi  r7, 4
        shr  r6, r7
        andi r6, 0x0F
        call print_nibble

        mov  r6, r0
        andi r6, 0x0F
        call print_nibble

        ret

; ── print_nibble ──────────────────────────────────────────────────────────────
; Convert r6 (0–15) to its hex character, write to FB at [r5] with attr ATTR,
; advance r5 by 2, and emit to serial.
; Clobbers: r6, r3.

print_nibble:
        cmpi r6, 10
        jlt  pn_digit
        addi r6, 55             ; 'A'–10 = 55  →  A–F
        jmp  pn_write
pn_digit:
        addi r6, 48             ; '0' = 48

pn_write:
        stb  [r5], r6
        addi r5, 1
        ldi  r3, ATTR
        stb  [r5], r3
        addi r5, 1
        out  r6, STDOUT
        ret
