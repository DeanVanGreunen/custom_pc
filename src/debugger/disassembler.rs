//! Disassembler: converts binary machine code back into assembly mnemonics.

use crate::memory::Memory;

/// Disassemble up to `count` instructions starting at `start_addr`.
/// Returns a list of `(address, bytes, mnemonic_string)` tuples.
pub fn disassemble(mem: &Memory, mut addr: u32, count: usize) -> Vec<(u32, Vec<u8>, String)> {
    let mut result = Vec::new();

    for _ in 0..count {
        let op = mem.read_byte(addr);

        let (text, len) = decode_text(mem, addr, op);

        let bytes: Vec<u8> = (0..len).map(|i| mem.read_byte(addr.wrapping_add(i as u32))).collect();
        result.push((addr, bytes, text));

        addr = addr.wrapping_add(len as u32);
    }

    result
}

fn decode_text(mem: &Memory, addr: u32, op: u8) -> (String, usize) {
    let rb   = || mem.read_byte(addr.wrapping_add(1));
    let imm  = || {
        let b0 = mem.read_byte(addr.wrapping_add(1)) as u32;
        let b1 = mem.read_byte(addr.wrapping_add(2)) as u32;
        let b2 = mem.read_byte(addr.wrapping_add(3)) as u32;
        let b3 = mem.read_byte(addr.wrapping_add(4)) as u32;
        b0 | (b1 << 8) | (b2 << 16) | (b3 << 24)
    };
    let ri   = || {
        let b  = mem.read_byte(addr.wrapping_add(1));
        let rd = b >> 4;
        let b0 = mem.read_byte(addr.wrapping_add(2)) as u32;
        let b1 = mem.read_byte(addr.wrapping_add(3)) as u32;
        let b2 = mem.read_byte(addr.wrapping_add(4)) as u32;
        let b3 = mem.read_byte(addr.wrapping_add(5)) as u32;
        (rd, b0 | (b1 << 8) | (b2 << 16) | (b3 << 24))
    };
    let rr   = || { let b = rb(); (b >> 4, b & 0x0F) };

    match op {
        0x00 => ("nop".into(), 1),
        0x01 => ("hlt".into(), 1),
        0x02 => { let (d,s)=rr(); (format!("mov  r{d}, r{s}"), 2) }
        0x03 => { let (d,i)=ri(); (format!("ldi  r{d}, 0x{i:08X}"), 6) }
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
        0x10 => { let (d,i)=ri(); (format!("addi r{d}, 0x{i:08X}"), 6) }
        0x11 => { let (d,i)=ri(); (format!("subi r{d}, 0x{i:08X}"), 6) }
        0x12 => { let (d,i)=ri(); (format!("andi r{d}, 0x{i:08X}"), 6) }
        0x13 => { let (d,i)=ri(); (format!("ori  r{d}, 0x{i:08X}"), 6) }
        0x14 => { let (d,i)=ri(); (format!("xori r{d}, 0x{i:08X}"), 6) }
        0x17 => { let (d,s)=rr(); (format!("mul  r{d}, r{s}"), 2) }
        0x18 => { let (d,s)=rr(); (format!("div  r{d}, r{s}"), 2) }
        0x19 => { let (d,s)=rr(); (format!("mod  r{d}, r{s}"), 2) }
        0x15 => { let (d,s)=rr(); (format!("cmp  r{d}, r{s}"), 2) }
        0x16 => { let (d,i)=ri(); (format!("cmpi r{d}, 0x{i:08X}"), 6) }
        0x20 => { let (_,s)=rr(); (format!("push r{s}"), 2) }
        0x21 => { let (d,_)=rr(); (format!("pop  r{d}"), 2) }
        0x22 => (format!("call 0x{:08X}", imm()), 5),
        0x23 => ("ret".into(), 1),
        0x30 => (format!("jmp  0x{:08X}", imm()), 5),
        0x31 => (format!("jz   0x{:08X}", imm()), 5),
        0x32 => (format!("jnz  0x{:08X}", imm()), 5),
        0x33 => (format!("jc   0x{:08X}", imm()), 5),
        0x34 => (format!("jnc  0x{:08X}", imm()), 5),
        0x35 => (format!("jn   0x{:08X}", imm()), 5),
        0x36 => (format!("jnn  0x{:08X}", imm()), 5),
        0x37 => (format!("jv   0x{:08X}", imm()), 5),
        0x38 => (format!("jnv  0x{:08X}", imm()), 5),
        0x39 => (format!("jgt  0x{:08X}", imm()), 5),
        0x3A => (format!("jlt  0x{:08X}", imm()), 5),
        0x3B => (format!("jge  0x{:08X}", imm()), 5),
        0x3C => (format!("jle  0x{:08X}", imm()), 5),
        0x40 => { let (d,i)=ri(); (format!("in   r{d}, 0x{i:08X}"), 6) }
        0x41 => { let (s,i)=ri(); (format!("out  r{s}, 0x{i:08X}"), 6) }
        0x50 => { let (d,s)=rr(); (format!("fadd  r{d}, r{s}"), 2) }
        0x51 => { let (d,s)=rr(); (format!("fsub  r{d}, r{s}"), 2) }
        0x52 => { let (d,s)=rr(); (format!("fmul  r{d}, r{s}"), 2) }
        0x53 => { let (d,s)=rr(); (format!("fdiv  r{d}, r{s}"), 2) }
        0x54 => { let (d,s)=rr(); (format!("fmod  r{d}, r{s}"), 2) }
        0x55 => { let (d,_)=rr(); (format!("fneg  r{d}"), 2) }
        0x56 => { let (d,_)=rr(); (format!("fabs  r{d}"), 2) }
        0x57 => { let (d,_)=rr(); (format!("fsqrt r{d}"), 2) }
        0x58 => { let (d,s)=rr(); (format!("fcmp  r{d}, r{s}"), 2) }
        0x59 => { let (d,_)=rr(); (format!("ftoi  r{d}"), 2) }
        0x5A => { let (d,_)=rr(); (format!("itof  r{d}"), 2) }
        0x5F => {
            let (d, bits) = ri();
            let f = f32::from_bits(bits);
            (format!("fldi  r{d}, {f:.6}"), 6)
        }
        other => (format!(".byte 0x{other:02X}"), 1),
    }
}
