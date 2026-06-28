# File Formats

---

## ROM Binary (.bin)

The assembler produces a flat raw binary. There is no header, no magic number, and no metadata — the file is a direct byte-for-byte image of the program starting at address 0x0000.

The emulator loads the binary by copying it into RAM at offset 0, then starts the CPU with PC = 0x0000.

### Producing a binary

```
cargo run -p assembler --bin asm -- <source.asm> <output.bin>
```

### Loading in the emulator

```
cargo run --bin custom_pc -- <output.bin>
```

---

## Assembly Source (.asm)

A plain-text file processed by the two-pass assembler. Encoding is UTF-8. Line endings may be LF or CRLF.

### Syntax overview

```asm
; This is a comment — everything after semicolon is ignored

        .equ NAME, value        ; define a named constant
        .org 0x0100             ; set the current address (pads with zeros)

label:                          ; define a label (resolves to current address)
        instruction operands    ; emit an instruction

data:
        .byte 0x41, 0x42, 0     ; emit raw bytes
        .string "Hello\n\0"     ; emit a UTF-8 string (escape sequences supported)
        .word 0x1234            ; emit a 16-bit little-endian word
```

### Labels

A label is a bare identifier followed by a colon on its own (or a shared) line. Labels resolve to their byte address in the output image and can be used anywhere an immediate or address is expected.

```asm
start:
        ldi  r0, message        ; r0 = address of 'message'
        call print
        hlt

message:
        .string "OK\n\0"
```

### Constants (.equ)

`.equ NAME, expression` binds a name to a numeric value. Constants are not assigned an address; they substitute their value wherever the name appears as an immediate.

```asm
        .equ STDOUT, 0x0000
        .equ LIMIT,  100
```

### Data directives

| Directive | Emits |
|-----------|-------|
| `.byte v1, v2, …` | One byte per argument (numbers or string literals) |
| `.string "…"` | Same as `.byte` with a string literal; supports `\n \t \0 \\ \"` |
| `.word v1, v2, …` | One 16-bit little-endian word per argument |
| `.org addr` | Advance the program counter to `addr` (cannot go backwards) |

### String escape sequences

| Escape | Value |
|--------|-------|
| `\n` | 0x0A (newline) |
| `\t` | 0x09 (tab) |
| `\0` | 0x00 (null) |
| `\\` | 0x5C (backslash) |
| `\"` | 0x22 (double quote) |

### Instruction operands

Registers are written `r0`–`r15`. Memory indirect operands use square brackets: `[rN]`. Immediates are decimal or `0x`-prefixed hex. Label names are valid wherever an immediate or address is expected.

```asm
        ldi  r0, 0xFF           ; immediate hex
        ldi  r1, 255            ; immediate decimal
        ldi  r2, label          ; label address as immediate
        ld   r3, [r2]           ; word load from address in r2
        stb  [r1], r0           ; byte store to address in r1
        call subroutine         ; call to label
        jmp  loop               ; jump to label
```

---

## Symbol Map (stderr output)

After assembly, the assembler prints a symbol table to stderr listing every label and constant in hex, sorted by name. This is informational only and not written to any file.

```
000A  ATTR
C000  FB_BASE
0000  STDOUT
0040  print_hex4
0077  print_nibble
assembled 158 bytes → roms/fibonacci.bin
```

The last line reports the total image size and output path.

---

## Debug / Disassembly

The interactive debugger (launched with `--debug`) can step through instructions and inspect registers and memory. It does not read or write any separate file format; it operates directly on the loaded binary in the emulator's RAM.
