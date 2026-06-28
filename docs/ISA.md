# Instruction Set Architecture

The custom CPU is a 16-bit RISC-style processor with a 16-bit address space (64 KiB addressable), 16 general-purpose registers, and a fixed-width instruction encoding of 1–4 bytes depending on the operand format.

---

## Registers

| Register | Name | Role |
|----------|------|------|
| r0–r13   | General purpose | Available to programs freely |
| r14      | SP (Stack Pointer) | Initialised to `0xFFFE` at reset; used by `push`/`pop`/`call`/`ret` |
| r15      | LR (Link Register) | Conventionally preserved; not automatically used by the ISA |

All registers are 16 bits wide. Values are unsigned unless an instruction treats them as signed (the signed comparison branches).

---

## Flags

The ALU maintains four status flags updated by most arithmetic and logic instructions.

| Flag | Symbol | Set when |
|------|--------|----------|
| Zero | Z | Result is `0x0000` |
| Carry | C | Unsigned addition overflowed bit 15, or subtraction `a >= b` (no borrow) |
| Negative | N | Bit 15 of the result is set (two's-complement sign bit) |
| Overflow | V | Signed result does not fit in 16 bits (sign-bit flip) |

`cmp`/`cmpi` set all four flags exactly as `sub` would but discard the result.  
`and`/`or`/`xor`/`not`/`shl`/`shr` update only Z and N.

---

## Instruction Encoding

Instructions are 1–4 bytes, loaded from address `0x0000` at reset.

| Format | Length | Layout |
|--------|--------|--------|
| `None` | 1 B | `[opcode]` |
| `Reg` | 2 B | `[opcode][0xr0]` — rd in high nibble, low nibble zero |
| `RegReg` | 2 B | `[opcode][rd:rs]` — each register packed into one nibble |
| `RegImm` | 4 B | `[opcode][rd:0][imm_lo][imm_hi]` — 16-bit immediate, little-endian |
| `Imm` | 3 B | `[opcode][addr_lo][addr_hi]` — 16-bit address, little-endian |

Memory operands (`ld`, `ldb`, `st`, `stb`) use `RegReg` encoding where one slot is the address register and the other is the data register.

---

## Opcode Table

### Control

| Mnemonic | Opcode | Format | Operation |
|----------|--------|--------|-----------|
| `nop` | 0x00 | None | No operation |
| `hlt` | 0x01 | None | Halt execution |

### Data Movement

| Mnemonic | Opcode | Format | Operation |
|----------|--------|--------|-----------|
| `mov rd, rs` | 0x02 | RegReg | `rd = rs` |
| `ldi rd, imm` | 0x03 | RegImm | `rd = imm` (16-bit immediate or label address) |
| `ld rd, [rs]` | 0x04 | RegReg | `rd = mem16[rs]` — load word, little-endian |
| `st [rd], rs` | 0x05 | RegReg | `mem16[rd] = rs` — store word, little-endian |
| `ldb rd, [rs]` | 0x06 | RegReg | `rd = mem8[rs]` — load byte (zero-extended) |
| `stb [rd], rs` | 0x07 | RegReg | `mem8[rd] = rs & 0xFF` — store low byte |

### ALU — Register

| Mnemonic | Opcode | Format | Flags | Operation |
|----------|--------|--------|-------|-----------|
| `add rd, rs` | 0x08 | RegReg | ZCNV | `rd = rd + rs` |
| `sub rd, rs` | 0x09 | RegReg | ZCNV | `rd = rd - rs` |
| `and rd, rs` | 0x0A | RegReg | ZN | `rd = rd & rs` |
| `or  rd, rs` | 0x0B | RegReg | ZN | `rd = rd \| rs` |
| `xor rd, rs` | 0x0C | RegReg | ZN | `rd = rd ^ rs` |
| `not rd`     | 0x0D | Reg    | ZN | `rd = ~rd` |
| `shl rd, rs` | 0x0E | RegReg | ZN | `rd = rd << (rs & 0xF)` — logical shift left |
| `shr rd, rs` | 0x0F | RegReg | ZN | `rd = rd >> (rs & 0xF)` — logical shift right |

### ALU — Immediate

| Mnemonic | Opcode | Format | Flags | Operation |
|----------|--------|--------|-------|-----------|
| `addi rd, imm` | 0x10 | RegImm | ZCNV | `rd = rd + imm` |
| `subi rd, imm` | 0x11 | RegImm | ZCNV | `rd = rd - imm` |
| `andi rd, imm` | 0x12 | RegImm | ZN | `rd = rd & imm` |
| `ori  rd, imm` | 0x13 | RegImm | ZN | `rd = rd \| imm` |
| `xori rd, imm` | 0x14 | RegImm | ZN | `rd = rd ^ imm` |

### Compare

| Mnemonic | Opcode | Format | Flags | Operation |
|----------|--------|--------|-------|-----------|
| `cmp  rd, rs`  | 0x15 | RegReg | ZCNV | Flags from `rd - rs`; result discarded |
| `cmpi rd, imm` | 0x16 | RegImm | ZCNV | Flags from `rd - imm`; result discarded |

### Stack and Subroutines

| Mnemonic | Opcode | Format | Operation |
|----------|--------|--------|-----------|
| `push rs` | 0x20 | Reg | `SP -= 2; mem16[SP] = rs` |
| `pop rd`  | 0x21 | Reg | `rd = mem16[SP]; SP += 2` |
| `call addr` | 0x22 | Imm | `push PC+3; PC = addr` |
| `ret`     | 0x23 | None | `pop PC` |

Stack grows downward. SP starts at `0xFFFE`. Stack overflow (SP wraps past `0xFFFD`) and underflow (SP past `0xFFFE`) are detected and halt the emulator with an error.

### Jumps

All jumps take a 16-bit absolute address.

| Mnemonic | Opcode | Condition |
|----------|--------|-----------|
| `jmp addr` | 0x30 | Unconditional |
| `jz  addr` | 0x31 | Z = 1 (result was zero / equal) |
| `jnz addr` | 0x32 | Z = 0 (result was non-zero / not equal) |
| `jc  addr` | 0x33 | C = 1 (unsigned carry / no borrow) |
| `jnc addr` | 0x34 | C = 0 (unsigned no-carry / borrow) |
| `jn  addr` | 0x35 | N = 1 (result negative) |
| `jnn addr` | 0x36 | N = 0 (result non-negative) |
| `jv  addr` | 0x37 | V = 1 (signed overflow) |
| `jnv addr` | 0x38 | V = 0 (no signed overflow) |
| `jgt addr` | 0x39 | !Z && (N == V) — signed greater-than |
| `jlt addr` | 0x3A | N != V — signed less-than |
| `jge addr` | 0x3B | N == V — signed greater-or-equal |
| `jle addr` | 0x3C | Z \|\| (N != V) — signed less-or-equal |

**Signed branch note:** `jgt`/`jlt`/`jge`/`jle` combine N and V instead of checking N alone. When a subtraction overflows, N gets the wrong polarity; comparing N == V cancels the overflow-induced sign flip and gives the correct signed result.

### I/O

| Mnemonic | Opcode | Format | Operation |
|----------|--------|--------|-----------|
| `in  rd, port`  | 0x40 | RegImm | `rd = bus.read(port)` |
| `out rs, port`  | 0x41 | RegImm | `bus.write(port, rs)` |

Port `0x0000` is the serial output. Writing a byte to it emits that ASCII character to the host terminal.
