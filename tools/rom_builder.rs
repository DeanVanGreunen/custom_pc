//! Combine multiple binary segments into a single ROM image.
//!
//! usage: rom_builder [--at <hex_addr> <file.bin>]... <output.bin>
//!
//! Example:
//!   rom_builder --at 0x0000 boot.bin --at 0x8000 kernel.bin rom.bin

use std::{fs, path::PathBuf, process};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: rom_builder [--at <addr> <file>]... <output>");
        process::exit(1);
    }

    let mut image = vec![0u8; 0x1_0000];
    let mut i = 0;
    let mut output: Option<PathBuf> = None;

    while i < args.len() {
        if args[i] == "--at" {
            if i + 2 >= args.len() {
                eprintln!("--at requires <addr> <file>");
                process::exit(1);
            }
            let addr_str = &args[i + 1];
            let file_str = &args[i + 2];
            let addr_str = addr_str.strip_prefix("0x")
                .or_else(|| addr_str.strip_prefix("0X"))
                .unwrap_or(addr_str);
            let addr = u16::from_str_radix(addr_str, 16).unwrap_or_else(|_| {
                eprintln!("invalid address '{}'", args[i + 1]);
                process::exit(1);
            }) as usize;

            let data = fs::read(file_str).unwrap_or_else(|e| {
                eprintln!("cannot read '{file_str}': {e}");
                process::exit(1);
            });

            let end = (addr + data.len()).min(0x1_0000);
            image[addr..end].copy_from_slice(&data[..end - addr]);
            eprintln!("  loaded {file_str} ({} bytes) at 0x{addr:04X}", data.len());
            i += 3;
        } else {
            // Last argument is the output path.
            output = Some(PathBuf::from(&args[i]));
            i += 1;
        }
    }

    let output = output.unwrap_or_else(|| { eprintln!("no output file specified"); process::exit(1); });

    // Trim trailing zeros.
    let last_nonzero = image.iter().rposition(|&b| b != 0).map_or(0, |p| p + 1);
    let trimmed = &image[..last_nonzero.max(1)];

    fs::write(&output, trimmed).unwrap_or_else(|e| {
        eprintln!("cannot write '{}': {e}", output.display());
        process::exit(1);
    });

    eprintln!("wrote {} bytes to {}", trimmed.len(), output.display());
}
