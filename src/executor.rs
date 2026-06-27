//! Instruction executor: mutates CPU state for one decoded instruction.

use crate::bus::Bus;
use crate::error::{EmuResult, EmulatorError};
use crate::flags::Flags;
use crate::instruction::Instruction;
use crate::memory::Memory;
use crate::registers::{Registers, SP};

/// Result of executing one instruction.
pub enum StepResult {
    /// Normal execution; new PC is provided.
    Continue(u16),
    /// CPU has halted; no further execution should occur.
    Halt,
}

pub fn execute(
    instr: Instruction,
    next_pc: u16,
    regs: &mut Registers,
    flags: &mut Flags,
    mem: &mut Memory,
    bus: &mut Bus,
) -> EmuResult<StepResult> {
    use Instruction::*;

    match instr {
        Nop => {}

        Hlt => return Ok(StepResult::Halt),

        Mov { rd, rs } => regs.set(rd, regs.get(rs)),

        Ldi { rd, imm } => regs.set(rd, imm),

        Ld { rd, rs } => {
            let addr = regs.get(rs);
            let val  = mem.read_word(addr)?;
            regs.set(rd, val);
        }
        St { rd, rs } => {
            let addr = regs.get(rd);
            let val  = regs.get(rs);
            mem.write_word(addr, val)?;
        }
        Ldb { rd, rs } => {
            let addr = regs.get(rs);
            regs.set(rd, mem.read_byte(addr) as u16);
        }
        Stb { rd, rs } => {
            let addr = regs.get(rd);
            mem.write_byte(addr, (regs.get(rs) & 0xFF) as u8);
        }

        Add { rd, rs } => {
            let a = regs.get(rd);
            let b = regs.get(rs);
            flags.set_add(a, b, 0);
            regs.set(rd, a.wrapping_add(b));
        }
        Sub { rd, rs } => {
            let a = regs.get(rd);
            let b = regs.get(rs);
            flags.set_sub(a, b);
            regs.set(rd, a.wrapping_sub(b));
        }
        And { rd, rs } => {
            let v = regs.get(rd) & regs.get(rs);
            flags.set_zn(v);
            regs.set(rd, v);
        }
        Or { rd, rs } => {
            let v = regs.get(rd) | regs.get(rs);
            flags.set_zn(v);
            regs.set(rd, v);
        }
        Xor { rd, rs } => {
            let v = regs.get(rd) ^ regs.get(rs);
            flags.set_zn(v);
            regs.set(rd, v);
        }
        Not { rd } => {
            let v = !regs.get(rd);
            flags.set_zn(v);
            regs.set(rd, v);
        }
        Shl { rd, rs } => {
            let shift = regs.get(rs) & 0xF;
            let v = regs.get(rd) << shift;
            flags.set_zn(v);
            regs.set(rd, v);
        }
        Shr { rd, rs } => {
            let shift = regs.get(rs) & 0xF;
            let v = regs.get(rd) >> shift;
            flags.set_zn(v);
            regs.set(rd, v);
        }

        Addi { rd, imm } => {
            let a = regs.get(rd);
            flags.set_add(a, imm, 0);
            regs.set(rd, a.wrapping_add(imm));
        }
        Subi { rd, imm } => {
            let a = regs.get(rd);
            flags.set_sub(a, imm);
            regs.set(rd, a.wrapping_sub(imm));
        }
        Andi { rd, imm } => {
            let v = regs.get(rd) & imm;
            flags.set_zn(v);
            regs.set(rd, v);
        }
        Ori { rd, imm } => {
            let v = regs.get(rd) | imm;
            flags.set_zn(v);
            regs.set(rd, v);
        }
        Xori { rd, imm } => {
            let v = regs.get(rd) ^ imm;
            flags.set_zn(v);
            regs.set(rd, v);
        }

        Cmp { rd, rs } => {
            flags.set_sub(regs.get(rd), regs.get(rs));
        }
        Cmpi { rd, imm } => {
            flags.set_sub(regs.get(rd), imm);
        }

        Push { rs } => {
            let sp = regs.get(SP).wrapping_sub(2);
            if sp > 0xFFFD {
                return Err(EmulatorError::StackOverflow);
            }
            regs.set(SP, sp);
            mem.write_word(sp, regs.get(rs))?;
        }
        Pop { rd } => {
            let sp = regs.get(SP);
            if sp > 0xFFFE {
                return Err(EmulatorError::StackUnderflow);
            }
            let val = mem.read_word(sp)?;
            regs.set(SP, sp.wrapping_add(2));
            regs.set(rd, val);
        }
        Call { addr } => {
            let sp = regs.get(SP).wrapping_sub(2);
            regs.set(SP, sp);
            mem.write_word(sp, next_pc)?;
            return Ok(StepResult::Continue(addr));
        }
        Ret => {
            let sp = regs.get(SP);
            let addr = mem.read_word(sp)?;
            regs.set(SP, sp.wrapping_add(2));
            return Ok(StepResult::Continue(addr));
        }

        Jmp { addr }  => return Ok(StepResult::Continue(addr)),
        Jz  { addr }  => if flags.zero     { return Ok(StepResult::Continue(addr)); }
        Jnz { addr }  => if !flags.zero    { return Ok(StepResult::Continue(addr)); }
        Jc  { addr }  => if flags.carry    { return Ok(StepResult::Continue(addr)); }
        Jnc { addr }  => if !flags.carry   { return Ok(StepResult::Continue(addr)); }
        Jn  { addr }  => if flags.negative { return Ok(StepResult::Continue(addr)); }
        Jnn { addr }  => if !flags.negative{ return Ok(StepResult::Continue(addr)); }
        Jv  { addr }  => if flags.overflow { return Ok(StepResult::Continue(addr)); }
        Jnv { addr }  => if !flags.overflow{ return Ok(StepResult::Continue(addr)); }

        // Signed comparisons use N and V.
        Jgt { addr } => if !flags.zero && (flags.negative == flags.overflow) { return Ok(StepResult::Continue(addr)); }
        Jlt { addr } => if flags.negative != flags.overflow                  { return Ok(StepResult::Continue(addr)); }
        Jge { addr } => if flags.negative == flags.overflow                  { return Ok(StepResult::Continue(addr)); }
        Jle { addr } => if flags.zero || (flags.negative != flags.overflow)  { return Ok(StepResult::Continue(addr)); }

        In  { rd, port } => {
            let val = bus.io_read(port)?;
            regs.set(rd, val);
        }
        Out { rs, port } => {
            bus.io_write(port, regs.get(rs))?;
        }
    }

    Ok(StepResult::Continue(next_pc))
}
