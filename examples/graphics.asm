; graphics.asm — pixel-mode demo: three horizontal colour bars on a 1024×720 display.
;
; Switches the display into pixel mode, then fills the framebuffer bank-by-bank:
;
;   Banks   0–59  (245 760 px)  →  red    (top 240 rows)
;   Banks  60–119 (245 760 px)  →  green  (middle 240 rows)
;   Banks 120–179 (245 760 px)  →  blue   (bottom 240 rows)
;
; Pixel format — 14-bit ARGB packed into a 16-bit word:
;   bits 13–12  Alpha  (0 = transparent … 3 = opaque)
;   bits 11– 8  Red    (0–15)
;   bits  7– 4  Green  (0–15)
;   bits  3– 0  Blue   (0–15)
;
; Control registers:
;   0xCFA0  REG_DISP_MODE  — write 1 to enable pixel mode
;   0xCFA1  REG_FB_BANK    — select which 8 KiB bank is visible (0–37)
;
; The 8 KiB window at 0xA000–0xBFFF holds 4 096 pixels (2 bytes each).
; Writing to this window updates the pixel at (bank * 4096 + offset/2).

; ── constants ─────────────────────────────────────────────────────────────────

        .equ REG_DISP_MODE, 0xCFA0
        .equ REG_FB_BANK,   0xCFA1
        .equ FB_WIN,        0xA000

        .equ BANK_COUNT,    180     ; ceil(1024*720 / 4096)
        .equ BANK_PIXELS,   4096    ; pixels per 8 KiB bank

        ; 14-bit ARGB: (A=3, R, G, B) → (3<<12)|(R<<8)|(G<<4)|B
        .equ PX_RED,        0x3F00  ; A=3 R=15 G=0  B=0
        .equ PX_GREEN,      0x30F0  ; A=3 R=0  G=15 B=0
        .equ PX_BLUE,       0x300F  ; A=3 R=0  G=0  B=15

        ; colour-band boundaries (bank index, inclusive start of next colour)
        .equ BAND1_START,   60
        .equ BAND2_START,   120

; ── enable pixel mode ─────────────────────────────────────────────────────────

        ldi  r0, REG_DISP_MODE
        ldi  r1, 1
        stb  [r0], r1               ; REG_DISP_MODE = 1

; ── outer loop: one bank per iteration ────────────────────────────────────────
;   r8 = current bank index (0 … BANK_COUNT-1)
;   r9 = pixel colour for this bank

        ldi  r8, 0

bank_loop:
        cmpi r8, BANK_COUNT
        jge  done

        ; ── select colour based on which band this bank falls in ──────────────
        cmpi r8, BAND1_START
        jlt  colour_red
        cmpi r8, BAND2_START
        jlt  colour_green
        ldi  r9, PX_BLUE
        jmp  set_bank
colour_red:
        ldi  r9, PX_RED
        jmp  set_bank
colour_green:
        ldi  r9, PX_GREEN

        ; ── write the bank register ───────────────────────────────────────────
set_bank:
        ldi  r0, REG_FB_BANK
        stb  [r0], r8               ; REG_FB_BANK = r8

        ; ── fill 4 096 pixels in the visible window ───────────────────────────
        ;   r2 = write pointer (walks 0xA000 … 0xBFFE, steps of 2)
        ;   r3 = pixels remaining

        ldi  r2, FB_WIN
        ldi  r3, BANK_PIXELS

pixel_loop:
        st   [r2], r9               ; write one 14-bit pixel word
        addi r2, 2
        subi r3, 1
        jnz  pixel_loop

        addi r8, 1
        jmp  bank_loop

; ── done ──────────────────────────────────────────────────────────────────────

done:
        hlt
