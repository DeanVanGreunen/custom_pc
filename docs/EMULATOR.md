# Emulator Reference

---

## Building

The workspace contains two crates: the emulator (`custom_pc`) and the assembler (`assembler`). Build both with:

```sh
cargo build --workspace
```

For an optimised build:

```sh
cargo build --workspace --release
```

---

## Running a ROM

```sh
cargo run --bin custom_pc -- <rom.bin>
```

Or with the release binary directly:

```sh
.\target\release\custom_pc.exe <rom.bin>
```

This opens a 1024×720 minifb window. The CPU runs at 200 000 instructions per rendered frame (approximately 12 MHz at 60 fps). The window closes when the CPU halts or the user presses Escape.

---

## Command-Line Flags

```
custom_pc <rom.bin> [--debug] [--max-cycles <n>]
```

| Flag | Description |
|------|-------------|
| `--debug` | Launch the interactive monitor instead of the windowed display |
| `--max-cycles <n>` | Stop execution after `n` total instructions |

---

## Interactive Debugger

Launch with `--debug` to enter the step-through monitor. No window is opened in this mode.

```sh
cargo run --bin custom_pc -- roms/fibonacci.bin --debug
```

The monitor accepts single-character commands:

| Command | Action |
|---------|--------|
| `s` | Step one instruction |
| `r` | Run until halt or breakpoint |
| `q` | Quit |

Register contents, flags, and the next instruction are printed after each step.

---

## Assembler

The assembler is a separate crate invoked as its own binary.

```sh
cargo run -p assembler --bin asm -- <source.asm> <output.bin>
```

**Examples — assembling all ROMs from the workspace root:**

```sh
cargo run -p assembler --bin asm -- examples/hello.asm     roms/hello.bin
cargo run -p assembler --bin asm -- examples/counter.asm   roms/counter.bin
cargo run -p assembler --bin asm -- examples/fibonacci.asm roms/fibonacci.bin
cargo run -p assembler --bin asm -- examples/stack.asm     roms/stack.bin
cargo run -p assembler --bin asm -- examples/graphics.asm  roms/graphics.bin
```

**PowerShell one-liner to assemble every `.asm` in `examples/`:**

```powershell
foreach ($f in Get-ChildItem examples\*.asm) {
    cargo run -p assembler --bin asm -- $f.FullName "roms\$($f.BaseName).bin"
}
```

The assembler prints a symbol map and byte count to stderr on success:

```
000A  ATTR
C000  FB_BASE
0000  STDOUT
0040  print_hex4
assembled 158 bytes → roms/fibonacci.bin
```

On error, it prints the source line number and a description, and exits with code 1.

---

## Example Programs

| Source | ROM | Description |
|--------|-----|-------------|
| `examples/hello.asm` | `roms/hello.bin` | Print "Hello, World!" to the text display and serial port |
| `examples/counter.asm` | `roms/counter.bin` | Count 0–9, writing each digit to the text display |
| `examples/fibonacci.asm` | `roms/fibonacci.bin` | First 16 Fibonacci numbers as 4-digit hex on the text display |
| `examples/stack.asm` | `roms/stack.bin` | Recursive factorial(3) using the call stack; result on text display |
| `examples/graphics.asm` | `roms/graphics.bin` | Three horizontal colour bars in pixel mode (1024×720) |

---

## Display Modes

### Text mode (default)

The window renders a 128×45 character grid from the text framebuffer at 0xC000. Each cell is 8×16 pixels (IBM CP437 font, row-doubled). No program action is required to enter text mode — it is active at reset.

### Pixel mode

Write `1` to `REG_DISP_MODE` (0xCFA0) to switch to pixel mode. Fill the framebuffer by selecting a bank via `REG_FB_BANK` (0xCFA1) and writing 14-bit ARGB pixel words into the 8 KiB window at 0xA000–0xBFFF.

```asm
        ldi  r0, 0xCFA0
        ldi  r1, 1
        stb  [r0], r1           ; enable pixel mode

        ldi  r0, 0xCFA1
        ldi  r8, 0
        stb  [r0], r8           ; select bank 0

        ldi  r2, 0xA000
        ldi  r9, 0x3F00         ; opaque red pixel
        st   [r2], r9           ; write pixel 0
```

See `docs/HARDWARE.md` for pixel format details and `docs/MEMORY_MAP.md` for the full address map.

---

## Emulator Architecture

```
src/
  main.rs          entry point, minifb window loop
  lib.rs           re-exports Machine
  machine.rs       Machine struct (CPU + Memory + Bus)
  cpu.rs           fetch/decode/execute loop
  decoder.rs       binary → Instruction
  executor.rs      Instruction → CPU state mutation
  flags.rs         ZCNV flag register
  registers.rs     16 × u16 register file
  instruction.rs   Instruction enum
  memory.rs        flat RAM + memory-mapped I/O routing
  bus.rs           I/O port dispatch
  error.rs         EmulatorError enum
  devices/
    display.rs     PixelDisplay + Display (text renderer, font, palette)
  debugger/
    monitor.rs     interactive step debugger

assembler/src/
  main.rs          CLI entry point
  lexer.rs         source text → tokens
  parser.rs        tokens → AST (Vec<Item>)
  ast.rs           Item / Operand / DataArg types
  codegen.rs       two-pass assembler → binary image
  symbol_table.rs  label and constant storage
  instruction.rs   opcode table and encoding formats
  error.rs         AsmError
```

---

## Conditional Jump Logic

The four signed comparison branches (`jgt`, `jlt`, `jge`, `jle`) combine the N and V flags rather than checking N alone. After `cmp rd, rs` (which computes `rd - rs`), a signed overflow can flip the N flag to the wrong polarity. Checking `N == V` cancels the flip and gives the correct signed result:

| Branch | Condition | Meaning after `cmp a, b` |
|--------|-----------|--------------------------|
| `jgt` | `!Z && (N == V)` | a > b (signed) |
| `jlt` | `N != V` | a < b (signed) |
| `jge` | `N == V` | a >= b (signed) |
| `jle` | `Z \|\| (N != V)` | a <= b (signed) |
