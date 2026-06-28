# Hardware Reference

This document describes the virtual hardware components of the Custom PC emulator.

---

## CPU

A 16-bit RISC processor clocked at an emulated rate of 200 000 instructions per rendered frame (approximately 12 MHz at 60 fps).

- 16 general-purpose 16-bit registers (r0–r15)
- r14 = Stack Pointer (SP), initialised to 0xFFFE at reset
- r15 = Link Register (LR), not automatically managed
- 4 status flags: Zero, Carry, Negative, Overflow
- 16-bit program counter (PC), starts at 0x0000
- Single address space: code and data share the same 64 KiB

See `docs/ISA.md` for the full instruction set.

---

## Memory

A flat byte-addressable array accessed through a memory controller that intercepts writes and reads to special address ranges before forwarding to the raw array.

- **RAM size:** configurable via `MEM_SIZE` in `src/memory.rs` (default 1 GiB; only the first 64 KiB is reachable by the 16-bit CPU)
- **Word access:** little-endian (low byte at lower address)
- **ROM:** no physical ROM; the program binary is loaded into RAM at 0x0000 at startup

---

## Display

The display outputs a 1024 × 720 pixel window rendered at up to 60 fps using minifb. Two modes share the same hardware registers.

### Text Mode (REG_DISP_MODE = 0, default)

A character-cell terminal rendered from the text framebuffer at 0xC000.

| Parameter | Value |
|-----------|-------|
| Resolution | 1024 × 720 px |
| Grid | 128 columns × 45 rows |
| Cell size | 8 × 16 px (8×8 font, rows doubled) |
| Font | IBM CP437 subset (0x20–0x7F), 8×8 bitmap, LSB = leftmost pixel |
| Colours | CGA 16-colour palette |

**CGA palette (index → 0xRRGGBB):**

```
0  #000000   Black               8  #555555   Dark grey
1  #0000AA   Dark blue           9  #5555FF   Bright blue
2  #00AA00   Dark green         10  #55FF55   Bright green
3  #00AAAA   Dark cyan          11  #55FFFF   Bright cyan
4  #AA0000   Dark red           12  #FF5555   Bright red
5  #AA00AA   Dark magenta       13  #FF55FF   Bright magenta
6  #AA5500   Brown              14  #FFFF55   Bright yellow
7  #AAAAAA   Light grey         15  #FFFFFF   White
```

**Attribute byte layout (byte 1 of each text cell):**

```
bit 7    reserved (zero)
bits 6–4 background colour index (0–7, dark colours only)
bits 3–0 foreground colour index (0–15, full palette)
```

Common attribute values:

```
0x07  white on black (default terminal)
0x0A  light green on black
0x0F  bright white on black
0x1F  bright white on dark blue
0x4E  bright yellow on dark red
```

### Pixel Mode (REG_DISP_MODE = 1)

Direct pixel access using a bank-switched framebuffer window.

| Parameter | Value |
|-----------|-------|
| Resolution | 1024 × 720 px |
| Total pixels | 737 280 |
| Bytes per pixel | 2 (14-bit ARGB packed into u16) |
| Backing store | 1 474 560 bytes |
| Bank window | 0xA000–0xBFFF (8 KiB = 4 096 pixels) |
| Bank count | 180 banks (0–179) |

**Pixel format (14-bit ARGB, u16, little-endian):**

```
bits 15–14  unused (should be zero)
bits 13–12  Alpha  (0 = transparent … 3 = opaque)
bits 11– 8  Red    (0–15)
bits  7– 4  Green  (0–15)
bits  3– 0  Blue   (0–15)
```

Example pixel values:

```
0x3F00  opaque red    (A=3, R=15, G=0,  B=0)
0x30F0  opaque green  (A=3, R=0,  G=15, B=0)
0x300F  opaque blue   (A=3, R=0,  G=0,  B=15)
0x3FFF  opaque white  (A=3, R=15, G=15, B=15)
0x0000  transparent black
```

**Bank addressing:**

To write pixel at screen position `(x, y)`:

```
pixel_index = y * 1024 + x
bank        = pixel_index / 4096
offset      = (pixel_index % 4096) * 2

; write low byte
stb  [0xA000 + offset],     low_byte
; write high byte
stb  [0xA000 + offset + 1], high_byte
```

---

## Display Control Registers

| Address | Register | Description |
|---------|----------|-------------|
| 0xCFA0 | REG_DISP_MODE | 0 = text mode, 1 = pixel mode |
| 0xCFA1 | REG_FB_BANK | Pixel bank select (0–179); reads back clamped value |
| 0xCFA2 | REG_DISP_CTRL | bit 0 = vsync enable; bit 1 = clear pixel FB (self-clearing) |

---

## I/O Bus

The I/O bus is separate from the address space and accessed with `in`/`out` instructions.

| Port | Direction | Device |
|------|-----------|--------|
| 0x0000 | Write | Serial output (emits byte as ASCII to host stdout) |
| 0x0000 | Read | Serial input (returns 0, not yet implemented) |

---

## Stack

The hardware stack uses r14 (SP) as a full-descending stack pointer.

- SP starts at 0xFFFE
- `push rs`: SP -= 2; mem[SP] = rs
- `pop rd`: rd = mem[SP]; SP += 2
- `call addr`: push return address (PC + instruction length); jump to addr
- `ret`: pop into PC

The emulator traps stack overflow (SP would go below 0xFFFC after decrement) and underflow (pop with SP >= 0xFFFE).
