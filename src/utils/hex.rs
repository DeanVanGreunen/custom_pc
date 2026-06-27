/// Format a byte slice as a hex dump (16 bytes per row).
pub fn hex_dump(data: &[u8], base_addr: u16) -> String {
    let mut out = String::new();
    for (i, chunk) in data.chunks(16).enumerate() {
        let addr = (base_addr as usize) + i * 16;
        out.push_str(&format!("{addr:04X}  "));
        for (j, &b) in chunk.iter().enumerate() {
            if j == 8 { out.push(' '); }
            out.push_str(&format!("{b:02X} "));
        }
        // Pad incomplete last row.
        for j in chunk.len()..16 {
            if j == 8 { out.push(' '); }
            out.push_str("   ");
        }
        out.push_str(" |");
        for &b in chunk {
            out.push(if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' });
        }
        out.push_str("|\n");
    }
    out
}

/// Format a `u16` as a zero-padded 4-digit hex string.
#[inline]
pub fn hex16(value: u16) -> String {
    format!("{value:04X}")
}

/// Format a `u8` as a zero-padded 2-digit hex string.
#[inline]
pub fn hex8(value: u8) -> String {
    format!("{value:02X}")
}
