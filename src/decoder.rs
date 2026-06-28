//! Binary → Instruction decoder.  Reads bytes from memory at the current PC.

use crate::error::{EmuResult, EmulatorError};
use crate::instruction::Instruction;
use crate::memory::Memory;
use crate::registers::Registers;

/// Decode the instruction at `pc`, returning the instruction and its byte length.
pub fn decode(mem: &Memory, pc: u32) -> EmuResult<(Instruction, u32)> {
    let op = mem.read_byte(pc);

    // Helper: read the packed register byte that follows the opcode.
    let reg_byte = || -> EmuResult<(u8, u8)> {
        Ok(Registers::unpack(mem.read_byte(pc.wrapping_add(1u32))))
    };
    // Helper: read a 32-bit immediate that starts after the opcode.
    let imm32 = |offset: u32| -> EmuResult<u32> {
        mem.read_word(pc.wrapping_add(offset))
    };
    // Helper: read the packed register byte + following 32-bit immediate.
    let reg_imm = || -> EmuResult<(u8, u32)> {
        let rb   = mem.read_byte(pc.wrapping_add(1u32));
        let (rd, _) = Registers::unpack(rb);
        let imm  = imm32(2)?;
        Ok((rd, imm))
    };

    let (instr, len) = match op {
        0x00 => (Instruction::Nop, 1),
        0x01 => (Instruction::Hlt, 1),

        0x02 => { let (rd, rs) = reg_byte()?; (Instruction::Mov { rd, rs }, 2) }
        0x03 => { let (rd, imm) = reg_imm()?; (Instruction::Ldi { rd, imm }, 6) }
        0x04 => { let (rd, rs) = reg_byte()?; (Instruction::Ld  { rd, rs }, 2) }
        0x05 => { let (rd, rs) = reg_byte()?; (Instruction::St  { rd, rs }, 2) }
        0x06 => { let (rd, rs) = reg_byte()?; (Instruction::Ldb { rd, rs }, 2) }
        0x07 => { let (rd, rs) = reg_byte()?; (Instruction::Stb { rd, rs }, 2) }

        0x08 => { let (rd, rs) = reg_byte()?; (Instruction::Add { rd, rs }, 2) }
        0x09 => { let (rd, rs) = reg_byte()?; (Instruction::Sub { rd, rs }, 2) }
        0x0A => { let (rd, rs) = reg_byte()?; (Instruction::And { rd, rs }, 2) }
        0x0B => { let (rd, rs) = reg_byte()?; (Instruction::Or  { rd, rs }, 2) }
        0x0C => { let (rd, rs) = reg_byte()?; (Instruction::Xor { rd, rs }, 2) }
        0x0D => { let (rd, _)  = reg_byte()?; (Instruction::Not { rd }, 2) }
        0x0E => { let (rd, rs) = reg_byte()?; (Instruction::Shl { rd, rs }, 2) }
        0x0F => { let (rd, rs) = reg_byte()?; (Instruction::Shr { rd, rs }, 2) }

        0x10 => { let (rd, imm) = reg_imm()?; (Instruction::Addi { rd, imm }, 6) }
        0x11 => { let (rd, imm) = reg_imm()?; (Instruction::Subi { rd, imm }, 6) }
        0x12 => { let (rd, imm) = reg_imm()?; (Instruction::Andi { rd, imm }, 6) }
        0x13 => { let (rd, imm) = reg_imm()?; (Instruction::Ori  { rd, imm }, 6) }
        0x14 => { let (rd, imm) = reg_imm()?; (Instruction::Xori { rd, imm }, 6) }
        0x17 => { let (rd, rs) = reg_byte()?; (Instruction::Mul  { rd, rs }, 2) }
        0x18 => { let (rd, rs) = reg_byte()?; (Instruction::Div  { rd, rs }, 2) }
        0x19 => { let (rd, rs) = reg_byte()?; (Instruction::Mod  { rd, rs }, 2) }
        0x15 => { let (rd, rs) = reg_byte()?; (Instruction::Cmp  { rd, rs }, 2) }
        0x16 => { let (rd, imm) = reg_imm()?; (Instruction::Cmpi { rd, imm }, 6) }

        0x20 => { let (_, rs) = reg_byte()?; (Instruction::Push { rs }, 2) }
        0x21 => { let (rd, _) = reg_byte()?; (Instruction::Pop  { rd }, 2) }
        0x22 => { let addr = imm32(1)?;       (Instruction::Call { addr }, 5) }
        0x23 => (Instruction::Ret, 1),

        0x30 => { let addr = imm32(1)?; (Instruction::Jmp { addr }, 5) }
        0x31 => { let addr = imm32(1)?; (Instruction::Jz  { addr }, 5) }
        0x32 => { let addr = imm32(1)?; (Instruction::Jnz { addr }, 5) }
        0x33 => { let addr = imm32(1)?; (Instruction::Jc  { addr }, 5) }
        0x34 => { let addr = imm32(1)?; (Instruction::Jnc { addr }, 5) }
        0x35 => { let addr = imm32(1)?; (Instruction::Jn  { addr }, 5) }
        0x36 => { let addr = imm32(1)?; (Instruction::Jnn { addr }, 5) }
        0x37 => { let addr = imm32(1)?; (Instruction::Jv  { addr }, 5) }
        0x38 => { let addr = imm32(1)?; (Instruction::Jnv { addr }, 5) }
        0x39 => { let addr = imm32(1)?; (Instruction::Jgt { addr }, 5) }
        0x3A => { let addr = imm32(1)?; (Instruction::Jlt { addr }, 5) }
        0x3B => { let addr = imm32(1)?; (Instruction::Jge { addr }, 5) }
        0x3C => { let addr = imm32(1)?; (Instruction::Jle { addr }, 5) }

        0x40 => { let (rd, port) = reg_imm()?; (Instruction::In  { rd, port }, 6) }
        0x41 => { let (rs, port) = reg_imm()?; (Instruction::Out { rs, port }, 6) }

        0x50 => { let (rd, rs) = reg_byte()?; (Instruction::Fadd  { rd, rs }, 2) }
        0x51 => { let (rd, rs) = reg_byte()?; (Instruction::Fsub  { rd, rs }, 2) }
        0x52 => { let (rd, rs) = reg_byte()?; (Instruction::Fmul  { rd, rs }, 2) }
        0x53 => { let (rd, rs) = reg_byte()?; (Instruction::Fdiv  { rd, rs }, 2) }
        0x54 => { let (rd, rs) = reg_byte()?; (Instruction::Fmod  { rd, rs }, 2) }
        0x55 => { let (rd, _)  = reg_byte()?; (Instruction::Fneg  { rd }, 2) }
        0x56 => { let (rd, _)  = reg_byte()?; (Instruction::Fabs  { rd }, 2) }
        0x57 => { let (rd, _)  = reg_byte()?; (Instruction::Fsqrt { rd }, 2) }
        0x58 => { let (rd, rs) = reg_byte()?; (Instruction::Fcmp  { rd, rs }, 2) }
        0x59 => { let (rd, _)  = reg_byte()?; (Instruction::Ftoi  { rd }, 2) }
        0x5A => { let (rd, _)  = reg_byte()?; (Instruction::Itof  { rd }, 2) }
        0x5F => { let (rd, imm) = reg_imm()?; (Instruction::Fldi  { rd, imm }, 6) }

        other => return Err(EmulatorError::InvalidOpcode(other)),
    };

    Ok((instr, len))
}
