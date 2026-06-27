; hello.asm — write "Hello, World!\n" to the text framebuffer and serial port.
;
; Text framebuffer: base 0xC000, 2 bytes per cell.
;   byte 0: ASCII character
;   byte 1: attribute  (0x0A = light green on black)

        .equ STDOUT,  0x0000
        .equ FB_BASE, 0xC000
        .equ ATTR,    0x0A

        ldi  r6, msg
        ldi  r5, FB_BASE

print_loop:
        ldb  r0, [r6]
        cmpi r0, 0
        jz   done

        stb  [r5], r0
        addi r5, 1
        ldi  r3, ATTR
        stb  [r5], r3
        addi r5, 1

        out  r0, STDOUT

        addi r6, 1
        jmp  print_loop

done:
        hlt

msg:
        .string "Hello, World!\n\0"
