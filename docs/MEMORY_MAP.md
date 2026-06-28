# Memory Map

The CPU has a 16-bit address space (0x0000–0xFFFF, 64 KiB). Addresses are routed by the memory controller before reaching the flat RAM array.

---

## Address Regions

| Range | Size | Description |
|-------|------|-------------|
| 0x0000–0x9FFF | 40 KiB | General-purpose RAM / ROM image |
| 0xA000–0xBFFF | 8 KiB | Pixel framebuffer bank window |
| 0xC000–0xCFFF | 4 KiB | Text framebuffer + I/O registers |
| 0xD000–0xFFFF | ~12 KiB | General-purpose RAM |

The CPU stack grows downward from SP = 0xFFFE. Keep stack usage away from any data or I/O registers in the upper range.

---

## 0x0000 — ROM / RAM

The assembler places the program image at 0x0000. ROM is not physically distinct from RAM in the emulator — the entire flat array is writable. A program is loaded by copying the binary into `data[0..]`.

---

## 0xA000–0xBFFF — Pixel Framebuffer Window

An 8 KiB (0x2000 byte) sliding window into the pixel display's backing store. The active region is selected by writing a bank index to `REG_FB_BANK` (0xCFA1).

- Each bank covers 4 096 pixels (8 192 bytes).
- The full 1024×720 display requires 180 banks (banks 0–179).
- Reads return the stored pixel data; writes update it.
- Effective pixel index: `bank * 4096 + (addr - 0xA000) / 2`

See `docs/HARDWARE.md` for the pixel format.

---

## 0xC000–0xCFBF — Text Framebuffer

128 columns × 45 rows = 5 760 cells. Each cell is 2 bytes:

```
byte 0: ASCII character code (0x00–0x7F; codes ≥ 0x80 are masked to 0x7F)
byte 1: attribute byte
          bits 3–0  foreground colour index (0–15, CGA palette)
          bits 6–4  background colour index (0–7, dark CGA colours only)
          bit  7    reserved / zero
```

Cell at column `c`, row `r`: address = `0xC000 + (r * 128 + c) * 2`

The text framebuffer is rendered whenever `REG_DISP_MODE` is 0 (the default). Characters in the IBM CP437 range 0x20–0x7F are displayed using an 8×8 bitmap font, row-doubled to 8×16 to fill each cell's 16-pixel height. Control codes (0x00–0x1F) render as blank cells.

---

## 0xCFA0–0xCFA2 — Display Control Registers

| Address | Name | Description |
|---------|------|-------------|
| 0xCFA0 | REG_DISP_MODE | Display mode: 0 = text (default), 1 = pixel |
| 0xCFA1 | REG_FB_BANK | Active pixel bank (0–179); clamped to valid range on write |
| 0xCFA2 | REG_DISP_CTRL | bit 0 = vsync-enable; bit 1 = clear trigger (self-clearing) |

Writing bit 1 of REG_DISP_CTRL clears the entire pixel framebuffer to zero and immediately clears the bit.

---

## 0xFFFC–0xFFFE — Stack

SP initialises to 0xFFFE. Each `push` or `call` decrements SP by 2 before writing. The deepest safe stack address (for a single word push) is 0xFFFC. The emulator raises `StackOverflow` if SP would wrap past 0xFFFD, and `StackUnderflow` if a `pop`/`ret` is attempted with SP at 0xFFFE or beyond.

---

## Serial Port (I/O Bus)

Serial I/O is not memory-mapped. Use the `out`/`in` instructions with port address 0x0000:

```asm
ldi  r0, 0x41        ; 'A'
out  r0, 0x0000      ; emit to serial
```

Output appears on the host terminal (stdout). There is no serial input device currently implemented (`in` returns 0).
