; hello.asm — write "Hello, World!\n" to the serial output (port 0x00).

        .equ STDOUT, 0x0000

        ldi  r15, msg       ; r15 = pointer into string

print_loop:
        ldb  r0, [r15]      ; load next character
        cmpi r0, 0          ; check for NUL terminator
        jz   done
        out  r0, STDOUT     ; emit character
        addi r15, 1
        jmp  print_loop

done:
        hlt

msg:
        .string "Hello, World!\n"
        .byte 0             ; NUL terminator
