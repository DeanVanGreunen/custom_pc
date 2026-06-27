use custom_pc::debugger::monitor::Monitor;
use custom_pc::devices::display::{Display, SCREEN_W, SCREEN_H};
use custom_pc::error::EmulatorError;
use custom_pc::Machine;
use minifb::{Key, Window, WindowOptions};
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
        return;
    }

    run_with_window(&mut machine, max_cycles);
}

/// Run the machine with a real-time minifb window showing the pixel display.
///
/// Each frame executes up to STEPS_PER_FRAME CPU instructions, then pushes
/// the pixel framebuffer to the window. The window stays open until the CPU
/// halts, the cycle limit is reached, or the user closes it.
fn run_with_window(machine: &mut Machine, max_cycles: Option<u64>) {
    let mut window = Window::new(
        "Custom PC",
        SCREEN_W,
        SCREEN_H,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        eprintln!("failed to create window: {e}");
        process::exit(1);
    });

    // Cap at ~60 fps so we don't burn CPU in the host for the render loop.
    window.set_target_fps(60);

    const STEPS_PER_FRAME: u64 = 200_000;
    let mut halted = false;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        if !halted {
            // Run up to STEPS_PER_FRAME more instructions this frame.
            // cpu.run() treats max_cycles as an absolute total-cycle ceiling,
            // so we compute the per-frame ceiling from the current cycle count.
            let frame_ceil = machine.cpu.cycles + STEPS_PER_FRAME;
            let limit = Some(match max_cycles {
                Some(global) => frame_ceil.min(global),
                None         => frame_ceil,
            });

            match machine.run(limit) {
                Ok(()) => {}
                Err(EmulatorError::Halted) => { halted = true; }
                Err(e) => {
                    eprintln!("runtime error: {e}");
                    halted = true;
                }
            }

            if machine.is_halted() { halted = true; }

            if let Some(global) = max_cycles {
                if machine.cpu.cycles >= global { halted = true; }
            }

            if halted {
                eprintln!("halted after {} cycles", machine.cpu.cycles);
            }
        }

        // Always push a frame so the window stays responsive even after halt.
        let buffer = if machine.mem.pixel_display.mode == 0 {
            Display::render_to_pixels(machine.mem.as_slice())
        } else {
            machine.mem.pixel_display.to_argb32()
        };
        if let Err(e) = window.update_with_buffer(&buffer, SCREEN_W, SCREEN_H) {
            eprintln!("window error: {e}");
            break;
        }
    }
}
