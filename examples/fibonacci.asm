; fibonacci.asm — compute the first 16 Fibonacci numbers and display each as
; a decimal number on the text framebuffer and serial port.
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
;   r6 = working copy of value to print  (print_dec argument)
;   r7 = leading-zero-suppressed flag  (print_dec scratch)
;   r8 = divisor temp  (print_dec scratch)
;   r9 = digit temp  (print_dec / pd_emit argument)
;   r3 = attribute byte  (subroutine scratch)
;   r4 = temp for fibonacci swap

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

        ; print r0 as decimal; pass value in r6 (leaves r0 intact)
        mov  r6, r0
        call print_dec

        ; write a space separator
        ldi  r9, 0x20
        stb  [r5], r9
        addi r5, 1
        ldi  r3, ATTR
        stb  [r5], r3
        addi r5, 1
        out  r9, STDOUT

        ; advance fibonacci: a, b = b, a+b
        mov  r4, r1
        add  r1, r0
        mov  r0, r4

        subi r2, 1
        jmp  fib_loop

done:
        hlt

; ── print_dec ─────────────────────────────────────────────────────────────────
; Print r6 (u32) as a decimal string with no leading zeros.
; If r6 == 0 the single character '0' is emitted.
; Advances r5 (text-FB pointer).  Clobbers: r3, r6, r7, r8, r9.

print_dec:
        ldi  r7, 0              ; r7 = printed_something flag

        ; ── ten-thousands digit ──────────────────────────────────────────────
        ldi  r8, 10000
        mov  r9, r6
        div  r9, r8             ; r9 = r6 / 10000
        mod  r6, r8             ; r6 = r6 % 10000
        cmpi r9, 0
        jnz  pd_e10k
        jmp  pd_thou
pd_e10k:
        ldi  r7, 1
        call pd_emit

        ; ── thousands digit ──────────────────────────────────────────────────
pd_thou:
        ldi  r8, 1000
        mov  r9, r6
        div  r9, r8
        mod  r6, r8
        cmpi r9, 0
        jnz  pd_ethou
        cmpi r7, 0
        jz   pd_hund
pd_ethou:
        ldi  r7, 1
        call pd_emit

        ; ── hundreds digit ───────────────────────────────────────────────────
pd_hund:
        ldi  r8, 100
        mov  r9, r6
        div  r9, r8
        mod  r6, r8
        cmpi r9, 0
        jnz  pd_ehund
        cmpi r7, 0
        jz   pd_tens
pd_ehund:
        ldi  r7, 1
        call pd_emit

        ; ── tens digit ───────────────────────────────────────────────────────
pd_tens:
        ldi  r8, 10
        mov  r9, r6
        div  r9, r8
        mod  r6, r8
        cmpi r9, 0
        jnz  pd_etens
        cmpi r7, 0
        jz   pd_ones
pd_etens:
        ldi  r7, 1
        call pd_emit

        ; ── ones digit (always emit) ─────────────────────────────────────────
pd_ones:
        mov  r9, r6
        call pd_emit

        ret

; ── pd_emit ───────────────────────────────────────────────────────────────────
; Write digit r9 (0–9) to the text FB at [r5] with attribute ATTR,
; advance r5 by 2, and emit the ASCII character to serial.
; Clobbers: r3, r9.

pd_emit:
        addi r9, 0x30
        stb  [r5], r9
        addi r5, 1
        ldi  r3, ATTR
        stb  [r5], r3
        addi r5, 1
        out  r9, STDOUT
        ret
