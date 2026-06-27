/// Extract a bit from a `u16` value.
#[inline]
pub fn bit(value: u16, pos: u8) -> bool {
    value & (1 << pos) != 0
}

/// Set or clear a bit in a `u16` value.
#[inline]
pub fn set_bit(value: u16, pos: u8, set: bool) -> u16 {
    if set { value | (1 << pos) } else { value & !(1 << pos) }
}

/// Sign-extend an `n`-bit value to `i16`.
#[inline]
pub fn sign_extend(value: u16, bits: u8) -> i16 {
    let shift = 16u8.saturating_sub(bits);
    ((value << shift) as i16) >> shift
}
