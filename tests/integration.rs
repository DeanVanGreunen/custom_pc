//! End-to-end integration tests: hand-encode a ROM and run it on the emulator.
//!
//! Run with:
//!   cargo test --test integration
//! or to see stdout from passing tests:
//!   cargo test --test integration -- --nocapture

use custom_pc::Machine;

fn run(rom: &[u8]) -> Machine {
    let mut m = Machine::new();
    m.load_rom(rom);
    m.run(Some(1_000_000)).unwrap();
    m
}

// ── encoding helpers ──────────────────────────────────────────────────────────
// All instructions use 32-bit immediates and addresses.
// Instruction sizes:
//   None   → 1 byte  (nop, hlt, ret)
//   Reg    → 2 bytes (opcode + reg_byte)
//   RegReg → 2 bytes (opcode + packed_regs)
//   RegImm → 6 bytes (opcode + reg_byte + 4-byte LE imm)
//   Imm    → 5 bytes (opcode + 4-byte LE imm)

fn pack(rd: u8, rs: u8) -> u8 { (rd << 4) | (rs & 0x0F) }
fn imm32(v: u32) -> [u8; 4] { v.to_le_bytes() }

// ldi rd, imm  →  6 bytes
fn ldi(rd: u8, v: u32) -> Vec<u8> {
    let [b0,b1,b2,b3] = imm32(v);
    vec![0x03, pack(rd, 0), b0, b1, b2, b3]
}
// addi rd, imm  →  6 bytes
fn addi(rd: u8, v: u32) -> Vec<u8> {
    let [b0,b1,b2,b3] = imm32(v);
    vec![0x10, pack(rd, 0), b0, b1, b2, b3]
}
// subi rd, imm  →  6 bytes
fn subi(rd: u8, v: u32) -> Vec<u8> {
    let [b0,b1,b2,b3] = imm32(v);
    vec![0x11, pack(rd, 0), b0, b1, b2, b3]
}
// st [rd], rs  →  2 bytes
fn st(rd: u8, rs: u8) -> Vec<u8> { vec![0x05, pack(rd, rs)] }
// ld rd, [rs]  →  2 bytes
fn ld(rd: u8, rs: u8)  -> Vec<u8> { vec![0x04, pack(rd, rs)] }
// mov rd, rs   →  2 bytes
fn mov(rd: u8, rs: u8) -> Vec<u8> { vec![0x02, pack(rd, rs)] }
// add rd, rs   →  2 bytes
fn add(rd: u8, rs: u8) -> Vec<u8> { vec![0x08, pack(rd, rs)] }
// jnz addr     →  5 bytes
fn jnz(addr: u32) -> Vec<u8> { let [b0,b1,b2,b3] = imm32(addr); vec![0x32, b0, b1, b2, b3] }
// hlt          →  1 byte
fn hlt() -> Vec<u8> { vec![0x01] }

fn rom(parts: &[Vec<u8>]) -> Vec<u8> { parts.iter().flatten().copied().collect() }

// ── tests ─────────────────────────────────────────────────────────────────────

#[test]
fn fibonacci_sequence() {
    // Compute the first 8 Fibonacci numbers into memory at 0x0200.
    // Words are 4 bytes each.  Layout:
    //   r0 = a (0), r1 = b (1), r2 = loop counter (8), r3 = write addr
    //
    //   ldi r0, 0          ; a = 0
    //   ldi r1, 1          ; b = 1
    //   ldi r2, 8          ; count = 8
    //   ldi r3, 0x0200     ; write pointer
    // loop:                ; address 24 (0x18)
    //   st  [r3], r0       ; mem[ptr] = a
    //   addi r3, 4         ; ptr += 4  (word = 4 bytes)
    //   mov  r4, r1        ; tmp = b
    //   add  r1, r0        ; b = a + b
    //   mov  r0, r4        ; a = tmp
    //   subi r2, 1         ; count--
    //   jnz  0x0018        ; loop while count != 0
    //   hlt

    let loop_addr: u32 = 24; // 4 × ldi (6 bytes each)

    let code = rom(&[
        ldi(0, 0),           // addr 0
        ldi(1, 1),           // addr 6
        ldi(2, 8),           // addr 12
        ldi(3, 0x0200),      // addr 18
        // loop @ 24:
        st(3, 0),            // addr 24
        addi(3, 4),          // addr 26
        mov(4, 1),           // addr 32
        add(1, 0),           // addr 34
        mov(0, 4),           // addr 36
        subi(2, 1),          // addr 38
        jnz(loop_addr),      // addr 44
        hlt(),               // addr 49
    ]);

    let m = run(&code);
    assert!(m.is_halted(), "CPU did not halt");

    let expected: [u32; 8] = [0, 1, 1, 2, 3, 5, 8, 13];
    for (i, &exp) in expected.iter().enumerate() {
        let addr = 0x0200u32 + (i as u32) * 4;
        let got = m.mem.read_word(addr).unwrap();
        assert_eq!(got, exp, "fib[{i}] at {addr:#06X}: expected {exp}, got {got}");
    }
}

#[test]
fn countdown_loop() {
    // Count down from 5 to 0.
    //   ldi r0, 5       ; addr 0
    // loop:             ; addr 6
    //   subi r0, 1      ; addr 6
    //   jnz  0x0006     ; addr 12
    //   hlt             ; addr 17

    let code = rom(&[
        ldi(0, 5),       // addr 0
        subi(0, 1),      // addr 6
        jnz(6),          // addr 12
        hlt(),           // addr 17
    ]);

    let m = run(&code);
    assert_eq!(m.cpu.regs.get(0), 0, "r0 should be 0 after countdown");
    assert!(m.cpu.flags.zero, "Z flag should be set after final subi");
}

#[test]
fn memory_roundtrip() {
    // Write a known word to memory with `st`, read it back with `ld`.
    //   ldi r0, 0xDEADBEEF   ; value to store
    //   ldi r1, 0x0400       ; address
    //   st  [r1], r0         ; write
    //   ldi r0, 0            ; clear r0
    //   ld  r0, [r1]         ; read back
    //   hlt

    let code = rom(&[
        ldi(0, 0xDEAD_BEEF),
        ldi(1, 0x0400),
        st(1, 0),
        ldi(0, 0),
        ld(0, 1),
        hlt(),
    ]);

    let m = run(&code);
    assert_eq!(m.cpu.regs.get(0), 0xDEAD_BEEF, "memory roundtrip failed");
}
