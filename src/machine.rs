//! Top-level machine: holds CPU, memory, and bus together.

use crate::bus::Bus;
use crate::cpu::{Cpu, CpuState};
use crate::error::EmuResult;
use crate::memory::Memory;

pub struct Machine {
    pub cpu: Cpu,
    pub mem: Memory,
    pub bus: Bus,
}

impl Machine {
    pub fn new() -> Self {
        Machine { cpu: Cpu::new(), mem: Memory::new(), bus: Bus::new() }
    }

    /// Load a binary ROM image into memory starting at address 0.
    pub fn load_rom(&mut self, image: &[u8]) {
        self.mem.load(image);
    }

    /// Reset the CPU to its initial state (memory is preserved).
    pub fn reset(&mut self) {
        self.cpu.reset();
    }

    pub fn is_halted(&self) -> bool {
        self.cpu.state == CpuState::Halted
    }

    /// Execute one instruction.
    pub fn step(&mut self) -> EmuResult<()> {
        self.cpu.step(&mut self.mem, &mut self.bus)
    }

    /// Run until HLT or the optional cycle limit.
    pub fn run(&mut self, max_cycles: Option<u64>) -> EmuResult<()> {
        self.cpu.run(&mut self.mem, &mut self.bus, max_cycles)
    }
}

impl Default for Machine {
    fn default() -> Self { Self::new() }
}
