//! Text-mode display: 80×25 character cells backed by a memory-mapped frame buffer.
//!
//! Frame buffer base address: 0xC000
//! Each cell is 2 bytes: [char_byte, attr_byte]
//! Total: 80 * 25 * 2 = 4000 bytes (ends at 0xCF9F)

pub const FB_BASE: u16    = 0xC000;
pub const FB_COLS: usize  = 80;
pub const FB_ROWS: usize  = 25;
pub const FB_SIZE: usize  = FB_COLS * FB_ROWS * 2;

/// Render the frame buffer from raw memory to a vector of (char, fg, bg) tuples.
pub struct Display;

impl Display {
    /// Read the frame buffer region from a flat memory slice and render each
    /// cell as `(character, foreground_color, background_color)`.
    pub fn render(mem: &[u8]) -> Vec<(char, u8, u8)> {
        let start = FB_BASE as usize;
        let end   = (start + FB_SIZE).min(mem.len());
        let buf   = &mem[start..end];

        buf.chunks_exact(2)
            .map(|cell| {
                let ch   = cell[0];
                let attr = cell[1];
                let fg   = attr & 0x0F;
                let bg   = (attr >> 4) & 0x07;
                let c    = if ch.is_ascii_graphic() || ch == b' ' { ch as char } else { ' ' };
                (c, fg, bg)
            })
            .collect()
    }

    /// Dump the frame buffer as plain text (newlines after every row).
    pub fn dump_text(mem: &[u8]) -> String {
        let cells = Self::render(mem);
        let mut out = String::new();
        for row in cells.chunks(FB_COLS) {
            for (c, _, _) in row {
                out.push(*c);
            }
            out.push('\n');
        }
        out
    }
}
