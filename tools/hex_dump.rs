use custom_pc::utils::hex::hex_dump;
use std::{fs, path::PathBuf, process};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: hex_dump <file.bin> [base_addr_hex]");
        process::exit(1);
    }

    let path = PathBuf::from(&args[1]);
    let base: u16 = args.get(2)
        .and_then(|s| {
            let s = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")).unwrap_or(s);
            u16::from_str_radix(s, 16).ok()
        })
        .unwrap_or(0);

    let data = fs::read(&path).unwrap_or_else(|e| {
        eprintln!("cannot read '{}': {e}", path.display());
        process::exit(1);
    });

    print!("{}", hex_dump(&data, base));
    eprintln!("{} bytes", data.len());
}
