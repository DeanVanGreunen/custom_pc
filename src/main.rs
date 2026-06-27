use custom_pc::debugger::monitor::Monitor;
use custom_pc::Machine;
use std::{fs, path::PathBuf, process};

fn usage() -> ! {
    eprintln!("usage: custom_pc <rom.bin> [--debug] [--max-cycles <n>]");
    process::exit(1);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 { usage(); }

    let rom_path   = PathBuf::from(&args[1]);
    let debug_mode = args.iter().any(|a| a == "--debug");
    let max_cycles = args.windows(2)
        .find(|w| w[0] == "--max-cycles")
        .and_then(|w| w[1].parse::<u64>().ok());

    let rom = fs::read(&rom_path).unwrap_or_else(|e| {
        eprintln!("cannot read '{}': {e}", rom_path.display());
        process::exit(1);
    });

    let mut machine = Machine::new();
    machine.load_rom(&rom);

    if debug_mode {
        Monitor::new().run(&mut machine);
    } else {
        if let Err(e) = machine.run(max_cycles) {
            use custom_pc::error::EmulatorError;
            match e {
                EmulatorError::Halted => {}
                other => { eprintln!("runtime error: {other}"); process::exit(1); }
            }
        }
        eprintln!("halted after {} cycles", machine.cpu.cycles);
    }
}
