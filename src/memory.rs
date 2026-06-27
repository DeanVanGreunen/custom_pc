//! 64 KiB flat address space with memory-mapped pixel display.
//!
//! Address routing:
//!   0xA000–0xBFFF → pixel framebuffer bank window  (PixelDisplay)
//!   0xCFA0        → REG_DISP_MODE
//!   0xCFA1        → REG_FB_BANK
//!   0xCFA2        → REG_DISP_CTRL  (bit 1 is self-clearing clear trigger)
//!   everything else → flat RAM/ROM array

use crate::devices::display::{
    PixelDisplay, FB_PIXEL_BASE, FB_PIXEL_END, FB_PIXEL_BANKS,
    REG_DISP_MODE, REG_FB_BANK, REG_DISP_CTRL,
};
use crate::error::{EmuResult, EmulatorError};

pub const MEM_SIZE: usize = 0x40_000_000;

#[derive(Clone)]
pub struct Memory {
    data: Vec<u8>,
    pub pixel_display: PixelDisplay,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            data: vec![0u8; MEM_SIZE],
            pixel_display: PixelDisplay::new(),
        }
    }

    /// Load a ROM image starting at address 0.
    pub fn load(&mut self, image: &[u8]) {
        let len = image.len().min(MEM_SIZE);
        self.data[..len].copy_from_slice(&image[..len]);
    }

    // ── byte access ──────────────────────────────────────────────────────────

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            FB_PIXEL_BASE..=FB_PIXEL_END => self.pixel_display.read_byte(addr),
            _ => self.data[addr as usize],
        }
    }

    pub fn write_byte(&mut self, addr: u16, value: u8) {
        match addr {
            FB_PIXEL_BASE..=FB_PIXEL_END => {
                self.pixel_display.write_byte(addr, value);
            }
            REG_DISP_MODE => {
                self.pixel_display.mode = value & 0x01;
                self.data[addr as usize] = self.pixel_display.mode;
            }
            REG_FB_BANK => {
                self.pixel_display.bank = value.min((FB_PIXEL_BANKS - 1) as u8);
                self.data[addr as usize] = self.pixel_display.bank;
            }
            REG_DISP_CTRL => {
                if value & 0x02 != 0 {
                    self.pixel_display.clear();
                }
                self.pixel_display.ctrl = value & !0x02;
                self.data[addr as usize] = self.pixel_display.ctrl;
            }
            _ => {
                self.data[addr as usize] = value;
            }
        }
    }

    // ── word access (little-endian) ──────────────────────────────────────────

    pub fn read_word(&self, addr: u16) -> EmuResult<u16> {
        if addr == 0xFFFF {
            return Err(EmulatorError::IllegalMemoryAccess(addr));
        }
        let lo = self.read_byte(addr) as u16;
        let hi = self.read_byte(addr + 1) as u16;
        Ok(lo | (hi << 8))
    }

    pub fn write_word(&mut self, addr: u16, value: u16) -> EmuResult<()> {
        if addr == 0xFFFF {
            return Err(EmulatorError::IllegalMemoryAccess(addr));
        }
        self.write_byte(addr,     (value & 0xFF) as u8);
        self.write_byte(addr + 1, (value >> 8)   as u8);
        Ok(())
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data[..]
    }
}

impl Default for Memory {
    fn default() -> Self { Self::new() }
}
