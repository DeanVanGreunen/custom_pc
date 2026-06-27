; stack.asm — demonstrate subroutine call/return and stack usage.

        ldi  r0, 3
        call factorial      ; r1 = 3!
        hlt

; factorial(r0) → result in r1.  Recursive implementation.
; Clobbers r0 on the way back (use push/pop to preserve).
factorial:
        cmpi r0, 1
        jle  base_case

        push r0             ; save n
        subi r0, 1
        call factorial      ; r1 = factorial(n-1)
        pop  r0             ; restore n
        ; r1 = (n-1)!  we need r1 = n * r1
        ; multiply r0 * r1 via repeated addition into r2
        ldi  r2, 0
        mov  r3, r0         ; loop counter = n
mul_loop:
        add  r2, r1
        subi r3, 1
        jnz  mul_loop
        mov  r1, r2
        ret

base_case:
        ldi  r1, 1
        ret
