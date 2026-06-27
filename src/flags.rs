//! CPU status flags register.

/// The four status flags produced by ALU operations.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Flags {
    /// Set when the result is zero.
    pub zero: bool,
    /// Set when an unsigned carry-out / borrow occurred.
    pub carry: bool,
    /// Set when the result has bit 15 set (negative in two's complement).
    pub negative: bool,
    /// Set when a signed overflow occurred.
    pub overflow: bool,
}

impl Flags {
    pub fn new() -> Self {
        Self::default()
    }

    /// Update Z and N from a 16-bit result.
    pub fn set_zn(&mut self, result: u16) {
        self.zero     = result == 0;
        self.negative = result & 0x8000 != 0;
    }

    /// Update Z, N, C, and V after an addition: `a + b (+ cin)`.
    pub fn set_add(&mut self, a: u16, b: u16, cin: u32) {
        let full = (a as u32) + (b as u32) + cin;
        let result = full as u16;
        self.zero     = result == 0;
        self.negative = result & 0x8000 != 0;
        self.carry    = full > 0xFFFF;
        // Overflow: same-sign inputs produced opposite-sign output.
        let sa = (a & 0x8000) != 0;
        let sb = (b & 0x8000) != 0;
        let sr = (result & 0x8000) != 0;
        self.overflow = (sa == sb) && (sa != sr);
    }

    /// Update Z, N, C, and V after a subtraction: `a - b`.
    pub fn set_sub(&mut self, a: u16, b: u16) {
        let full = (a as u32).wrapping_add((!b as u32).wrapping_add(1));
        let result = full as u16;
        self.zero     = result == 0;
        self.negative = result & 0x8000 != 0;
        self.carry    = a >= b; // borrow = !carry
        let sa = (a & 0x8000) != 0;
        let sb = (b & 0x8000) != 0;
        let sr = (result & 0x8000) != 0;
        self.overflow = (sa != sb) && (sa != sr);
    }
}
