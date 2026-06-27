//! 64 KiB flat address space.

use crate::error::{EmuResult, EmulatorError};

pub const MEM_SIZE: usize = 0x1_0000;

#[derive(Clone)]
pub struct Memory {
    data: Box<[u8; MEM_SIZE]>,
}

impl Memory {
    pub fn new() -> Self {
        Memory { data: Box::new([0u8; MEM_SIZE]) }
    }

    /// Load a ROM image starting at address 0.
    pub fn load(&mut self, image: &[u8]) {
        let len = image.len().min(MEM_SIZE);
        self.data[..len].copy_from_slice(&image[..len]);
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        self.data[addr as usize]
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        self.data[addr as usize] = value;
    }

    pub fn read_word(&self, addr: u16) -> EmuResult<u16> {
        if addr == 0xFFFF {
            return Err(EmulatorError::IllegalMemoryAccess(addr));
        }
        let lo = self.data[addr as usize] as u16;
        let hi = self.data[(addr as usize) + 1] as u16;
        Ok(lo | (hi << 8))
    }

    pub fn write_word(&mut self, addr: u16, value: u16) -> EmuResult<()> {
        if addr == 0xFFFF {
            return Err(EmulatorError::IllegalMemoryAccess(addr));
        }
        self.data[addr as usize]       = (value & 0xFF) as u8;
        self.data[(addr as usize) + 1] = (value >> 8) as u8;
        Ok(())
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}
