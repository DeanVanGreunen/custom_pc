//! CPU core: fetch → decode → execute loop.

use crate::bus::Bus;
use crate::decoder::decode;
use crate::error::{EmuResult, EmulatorError};
use crate::executor::{execute, StepResult};
use crate::flags::Flags;
use crate::memory::Memory;
use crate::registers::Registers;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuState {
    Running,
    Halted,
}

pub struct Cpu {
    pub regs:  Registers,
    pub flags: Flags,
    pub pc:    u32,
    pub state: CpuState,
    /// Total instructions executed (useful for debugging and tests).
    pub cycles: u64,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            regs:   Registers::new(),
            flags:  Flags::new(),
            pc:     0x0000u32,
            state:  CpuState::Running,
            cycles: 0,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn is_halted(&self) -> bool {
        self.state == CpuState::Halted
    }

    /// Execute one instruction. Returns an error if the CPU is already halted
    /// or an invalid opcode / bad memory access is encountered.
    pub fn step(&mut self, mem: &mut Memory, bus: &mut Bus) -> EmuResult<()> {
        if self.state == CpuState::Halted {
            return Err(EmulatorError::Halted);
        }

        let (instr, len) = decode(mem, self.pc)?;
        let next_pc = self.pc.wrapping_add(len);

        match execute(instr, next_pc, &mut self.regs, &mut self.flags, mem, bus)? {
            StepResult::Continue(new_pc) => self.pc = new_pc,
            StepResult::Halt => {
                self.pc    = next_pc;
                self.state = CpuState::Halted;
            }
        }

        self.cycles += 1;
        Ok(())
    }

    /// Run until `HLT`, a step limit, or an error.
    pub fn run(&mut self, mem: &mut Memory, bus: &mut Bus, max_cycles: Option<u64>) -> EmuResult<()> {
        loop {
            if self.state == CpuState::Halted { return Ok(()); }
            if let Some(limit) = max_cycles {
                if self.cycles >= limit { return Ok(()); }
            }
            self.step(mem, bus)?;
        }
    }
}

impl Default for Cpu {
    fn default() -> Self { Self::new() }
}
