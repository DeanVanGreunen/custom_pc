//! Disassembler: converts binary machine code back into assembly mnemonics.

use crate::memory::Memory;

/// Disassemble up to `count` instructions starting at `start_addr`.
/// Returns a list of `(address, bytes, mnemonic_string)` tuples.
pub fn disassemble(mem: &Memory, mut addr: u16, count: usize) -> Vec<(u16, Vec<u8>, String)> {
    let mut result = Vec::new();

    for _ in 0..count {
        let op = mem.read_byte(addr);

        let (text, len) = decode_text(mem, addr, op);

        let bytes: Vec<u8> = (0..len).map(|i| mem.read_byte(addr.wrapping_add(i as u16))).collect();
        result.push((addr, bytes, text));

        addr = addr.wrapping_add(len as u16);
    }

    result
}

fn decode_text(mem: &Memory, addr: u16, op: u8) -> (String, usize) {
    let rb   = || mem.read_byte(addr.wrapping_add(1));
    let imm  = || {
        let lo = mem.read_byte(addr.wrapping_add(1)) as u16;
        let hi = mem.read_byte(addr.wrapping_add(2)) as u16;
        lo | (hi << 8)
    };
    let ri   = || {
        let b  = mem.read_byte(addr.wrapping_add(1));
        let rd = b >> 4;
        let lo = mem.read_byte(addr.wrapping_add(2)) as u16;
        let hi = mem.read_byte(addr.wrapping_add(3)) as u16;
        (rd, lo | (hi << 8))
    };
    let rr   = || { let b = rb(); (b >> 4, b & 0x0F) };

    match op {
        0x00 => ("nop".into(), 1),
        0x01 => ("hlt".into(), 1),
        0x02 => { let (d,s)=rr(); (format!("mov  r{d}, r{s}"), 2) }
        0x03 => { let (d,i)=ri(); (format!("ldi  r{d}, 0x{i:04X}"), 4) }
        0x04 => { let (d,s)=rr(); (format!("ld   r{d}, [r{s}]"), 2) }
        0x05 => { let (d,s)=rr(); (format!("st   [r{d}], r{s}"), 2) }
        0x06 => { let (d,s)=rr(); (format!("ldb  r{d}, [r{s}]"), 2) }
        0x07 => { let (d,s)=rr(); (format!("stb  [r{d}], r{s}"), 2) }
        0x08 => { let (d,s)=rr(); (format!("add  r{d}, r{s}"), 2) }
        0x09 => { let (d,s)=rr(); (format!("sub  r{d}, r{s}"), 2) }
        0x0A => { let (d,s)=rr(); (format!("and  r{d}, r{s}"), 2) }
        0x0B => { let (d,s)=rr(); (format!("or   r{d}, r{s}"), 2) }
        0x0C => { let (d,s)=rr(); (format!("xor  r{d}, r{s}"), 2) }
        0x0D => { let (d,_)=rr(); (format!("not  r{d}"), 2) }
        0x0E => { let (d,s)=rr(); (format!("shl  r{d}, r{s}"), 2) }
        0x0F => { let (d,s)=rr(); (format!("shr  r{d}, r{s}"), 2) }
        0x10 => { let (d,i)=ri(); (format!("addi r{d}, 0x{i:04X}"), 4) }
        0x11 => { let (d,i)=ri(); (format!("subi r{d}, 0x{i:04X}"), 4) }
        0x12 => { let (d,i)=ri(); (format!("andi r{d}, 0x{i:04X}"), 4) }
        0x13 => { let (d,i)=ri(); (format!("ori  r{d}, 0x{i:04X}"), 4) }
        0x14 => { let (d,i)=ri(); (format!("xori r{d}, 0x{i:04X}"), 4) }
        0x15 => { let (d,s)=rr(); (format!("cmp  r{d}, r{s}"), 2) }
        0x16 => { let (d,i)=ri(); (format!("cmpi r{d}, 0x{i:04X}"), 4) }
        0x20 => { let (_,s)=rr(); (format!("push r{s}"), 2) }
        0x21 => { let (d,_)=rr(); (format!("pop  r{d}"), 2) }
        0x22 => (format!("call 0x{:04X}", imm()), 3),
        0x23 => ("ret".into(), 1),
        0x30 => (format!("jmp  0x{:04X}", imm()), 3),
        0x31 => (format!("jz   0x{:04X}", imm()), 3),
        0x32 => (format!("jnz  0x{:04X}", imm()), 3),
        0x33 => (format!("jc   0x{:04X}", imm()), 3),
        0x34 => (format!("jnc  0x{:04X}", imm()), 3),
        0x35 => (format!("jn   0x{:04X}", imm()), 3),
        0x36 => (format!("jnn  0x{:04X}", imm()), 3),
        0x37 => (format!("jv   0x{:04X}", imm()), 3),
        0x38 => (format!("jnv  0x{:04X}", imm()), 3),
        0x39 => (format!("jgt  0x{:04X}", imm()), 3),
        0x3A => (format!("jlt  0x{:04X}", imm()), 3),
        0x3B => (format!("jge  0x{:04X}", imm()), 3),
        0x3C => (format!("jle  0x{:04X}", imm()), 3),
        0x40 => { let (d,i)=ri(); (format!("in   r{d}, 0x{i:04X}"), 4) }
        0x41 => { let (s,i)=ri(); (format!("out  r{s}, 0x{i:04X}"), 4) }
        other => (format!(".byte 0x{other:02X}"), 1),
    }
}
