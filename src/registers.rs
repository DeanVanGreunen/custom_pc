//! General-purpose register file: r0–r15 (32-bit each).
//!
//! r14 is the stack pointer (SP); r15 is the link register (LR).

pub const SP: u8 = 14;
pub const LR: u8 = 15;

/// Stack pointer initial value: top of the 1 GiB address space.
pub const SP_INIT: u32 = 0x3FFF_FFFC;

#[derive(Debug, Clone)]
pub struct Registers([u32; 16]);

impl Registers {
    pub fn new() -> Self {
        let mut r = Registers([0u32; 16]);
        r.0[SP as usize] = SP_INIT;
        r
    }

    #[inline]
    pub fn get(&self, index: u8) -> u32 {
        self.0[(index & 0x0F) as usize]
    }

    #[inline]
    pub fn set(&mut self, index: u8, value: u32) {
        self.0[(index & 0x0F) as usize] = value;
    }

    /// Decode a packed register byte: high nibble = rd, low nibble = rs.
    #[inline]
    pub fn unpack(byte: u8) -> (u8, u8) {
        (byte >> 4, byte & 0x0F)
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}
