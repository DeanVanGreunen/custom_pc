use custom_pc::decoder::decode;
use custom_pc::instruction::Instruction;
use custom_pc::memory::Memory;

fn mem_from(bytes: &[u8]) -> Memory {
    let mut m = Memory::new();
    m.load(bytes);
    m
}

#[test]
fn decode_nop() {
    let mem = mem_from(&[0x00]);
    let (instr, len) = decode(&mem, 0).unwrap();
    assert_eq!(instr, Instruction::Nop);
    assert_eq!(len, 1);
}

#[test]
fn decode_hlt() {
    let mem = mem_from(&[0x01]);
    let (instr, len) = decode(&mem, 0).unwrap();
    assert_eq!(instr, Instruction::Hlt);
    assert_eq!(len, 1);
}

#[test]
fn decode_ldi() {
    // ldi r0, 0x1234  → 0x03 0x00 0x34 0x12
    let mem = mem_from(&[0x03, 0x00, 0x34, 0x12]);
    let (instr, len) = decode(&mem, 0).unwrap();
    assert_eq!(instr, Instruction::Ldi { rd: 0, imm: 0x1234 });
    assert_eq!(len, 4);
}

#[test]
fn decode_jmp() {
    // jmp 0x0004 → 0x30 0x04 0x00
    let mem = mem_from(&[0x30, 0x04, 0x00]);
    let (instr, len) = decode(&mem, 0).unwrap();
    assert_eq!(instr, Instruction::Jmp { addr: 0x0004 });
    assert_eq!(len, 3);
}

#[test]
fn decode_ld_st() {
    // ld r1, [r2] → 0x04 0x12
    let mem = mem_from(&[0x04, 0x12, 0x05, 0x34]);
    let (instr, len) = decode(&mem, 0).unwrap();
    assert_eq!(instr, Instruction::Ld { rd: 1, rs: 2 });
    assert_eq!(len, 2);

    let (instr, len) = decode(&mem, 2).unwrap();
    assert_eq!(instr, Instruction::St { rd: 3, rs: 4 });
    assert_eq!(len, 2);
}

#[test]
fn decode_invalid_opcode() {
    let mem = mem_from(&[0xFF]);
    assert!(decode(&mem, 0).is_err());
}
