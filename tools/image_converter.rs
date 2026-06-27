//! Convert between raw binary ROM images and Intel HEX (.hex) format.
//!
//! usage: image_converter <input> <output>
//!   Converts .bin → .hex or .hex → .bin based on file extensions.

use std::{fs, path::PathBuf, process};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: image_converter <input> <output>");
        process::exit(1);
    }

    let input  = PathBuf::from(&args[1]);
    let output = PathBuf::from(&args[2]);

    let in_ext  = input.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let out_ext = output.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

    match (in_ext.as_str(), out_ext.as_str()) {
        ("bin", "hex") => {
            let data = read_file(&input);
            let hex  = bin_to_ihex(&data);
            fs::write(&output, hex).unwrap_or_else(|e| die(&format!("write: {e}")));
        }
        ("hex", "bin") => {
            let text = fs::read_to_string(&input)
                .unwrap_or_else(|e| die(&format!("read: {e}")));
            let data = ihex_to_bin(&text);
            fs::write(&output, data).unwrap_or_else(|e| die(&format!("write: {e}")));
        }
        _ => die("unsupported conversion; use .bin↔.hex"),
    }

    eprintln!("converted {} → {}", input.display(), output.display());
}

fn read_file(path: &PathBuf) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|e| die(&format!("cannot read '{}': {e}", path.display())))
}

fn die(msg: &str) -> ! { eprintln!("{msg}"); process::exit(1); }

/// Convert a binary slice to Intel HEX records (16-byte data records + EOF).
fn bin_to_ihex(data: &[u8]) -> String {
    let mut out = String::new();
    for (i, chunk) in data.chunks(16).enumerate() {
        let addr = (i * 16) as u16;
        let mut rec: Vec<u8> = Vec::new();
        rec.push(chunk.len() as u8);
        rec.push((addr >> 8) as u8);
        rec.push((addr & 0xFF) as u8);
        rec.push(0x00); // data record type
        rec.extend_from_slice(chunk);
        let checksum = (0u8).wrapping_sub(rec.iter().fold(0u8, |a, &b| a.wrapping_add(b)));
        rec.push(checksum);
        out.push(':');
        for b in &rec { out.push_str(&format!("{b:02X}")); }
        out.push('\n');
    }
    out.push_str(":00000001FF\n"); // EOF record
    out
}

/// Parse Intel HEX records into a binary image.
fn ihex_to_bin(text: &str) -> Vec<u8> {
    let mut image = vec![0u8; 0x1_0000];
    let mut last_addr = 0usize;

    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with(':') { continue; }
        let bytes: Vec<u8> = line[1..]
            .as_bytes()
            .chunks(2)
            .filter_map(|c| {
                let s = std::str::from_utf8(c).ok()?;
                u8::from_str_radix(s, 16).ok()
            })
            .collect();
        if bytes.len() < 5 { continue; }
        let byte_count = bytes[0] as usize;
        let addr       = ((bytes[1] as usize) << 8) | bytes[2] as usize;
        let rec_type   = bytes[3];
        if rec_type == 0x01 { break; } // EOF
        if rec_type != 0x00 { continue; }
        let data = &bytes[4..4 + byte_count.min(bytes.len().saturating_sub(5))];
        let end = (addr + data.len()).min(0x1_0000);
        image[addr..end].copy_from_slice(&data[..end - addr]);
        last_addr = last_addr.max(end);
    }

    image[..last_addr].to_vec()
}
