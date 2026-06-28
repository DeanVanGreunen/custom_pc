//! Interactive monitor: a simple REPL for inspecting and controlling the machine.

use std::io::{self, BufRead, Write};
use crate::debugger::breakpoint::Breakpoints;
use crate::debugger::disassembler::disassemble;
use crate::machine::Machine;
use crate::utils::hex::hex_dump;

pub struct Monitor {
    pub breakpoints: Breakpoints,
}

impl Monitor {
    pub fn new() -> Self {
        Monitor { breakpoints: Breakpoints::new() }
    }

    /// Run the interactive REPL.  Returns when the user types `quit`.
    pub fn run(&mut self, machine: &mut Machine) {
        let stdin  = io::stdin();
        let stdout = io::stdout();

        println!("custom_pc monitor — type 'help' for commands");

        loop {
            print!("> ");
            stdout.lock().flush().ok();

            let mut line = String::new();
            if stdin.lock().read_line(&mut line).is_err() { break; }
            let line = line.trim();
            if line.is_empty() { continue; }

            let mut parts = line.splitn(2, ' ');
            let cmd = parts.next().unwrap_or("");
            let arg = parts.next().unwrap_or("").trim();

            match cmd {
                "help" | "h" | "?" => {
                    println!("  step  [n]          — execute n instructions (default 1)");
                    println!("  run                — run until HLT or breakpoint");
                    println!("  regs               — dump registers");
                    println!("  dis   [addr] [n]   — disassemble n instructions at addr");
                    println!("  mem   <addr> [n]   — hex dump n bytes at addr");
                    println!("  bp    <addr>        — add breakpoint");
                    println!("  bpd   <addr>        — delete breakpoint");
                    println!("  bpl                — list breakpoints");
                    println!("  reset              — reset CPU");
                    println!("  quit               — exit");
                }
                "step" | "s" => {
                    let n: u64 = arg.parse().unwrap_or(1);
                    for _ in 0..n {
                        if let Err(e) = machine.step() {
                            println!("  {e}"); break;
                        }
                        if machine.is_halted() { println!("  halted"); break; }
                        if self.breakpoints.is_set(machine.cpu.pc) {
                            println!("  breakpoint at 0x{:08X}", machine.cpu.pc);
                            break;
                        }
                    }
                    self.print_regs(machine);
                }
                "run" | "r" => {
                    loop {
                        if let Err(e) = machine.step() {
                            println!("  {e}"); break;
                        }
                        if machine.is_halted() { println!("  halted"); break; }
                        if self.breakpoints.is_set(machine.cpu.pc) {
                            println!("  breakpoint at 0x{:08X}", machine.cpu.pc);
                            break;
                        }
                    }
                    self.print_regs(machine);
                }
                "regs" => self.print_regs(machine),
                "dis"  => {
                    let (addr_s, rest) = arg.split_once(' ').unwrap_or((arg, ""));
                    let addr = parse_u32(addr_s).unwrap_or(machine.cpu.pc);
                    let n    = rest.trim().parse::<usize>().unwrap_or(8);
                    for (a, bytes, text) in disassemble(&machine.mem, addr, n) {
                        let hex: String = bytes.iter().map(|b| format!("{b:02X} ")).collect();
                        println!("  {:08X}  {:12}{}", a, hex, text);
                    }
                }
                "mem"  => {
                    let (addr_s, rest) = arg.split_once(' ').unwrap_or((arg, ""));
                    let addr = parse_u32(addr_s).unwrap_or(0);
                    let n    = rest.trim().parse::<usize>().unwrap_or(64);
                    let slice: Vec<u8> = (0..n as u32)
                        .map(|i| machine.mem.read_byte(addr.wrapping_add(i)))
                        .collect();
                    print!("{}", hex_dump(&slice, addr));
                }
                "bp"   => if let Some(a) = parse_u32(arg) {
                    self.breakpoints.add(a);
                    println!("  breakpoint added at 0x{a:08X}");
                }
                "bpd"  => if let Some(a) = parse_u32(arg) {
                    self.breakpoints.remove(a);
                    println!("  breakpoint removed");
                }
                "bpl"  => {
                    for a in self.breakpoints.list() {
                        println!("  0x{a:08X}");
                    }
                }
                "reset" => { machine.reset(); println!("  CPU reset"); }
                "quit" | "q" | "exit" => break,
                other => println!("  unknown command '{other}'"),
            }
        }
    }

    fn print_regs(&self, machine: &Machine) {
        let cpu = &machine.cpu;
        for row in 0..4 {
            for col in 0..4 {
                let i = row * 4 + col;
                print!("  r{i:<2} = 0x{:08X}", cpu.regs.get(i as u8));
            }
            println!();
        }
        let f = &cpu.flags;
        println!("  PC=0x{:08X}  Z={} C={} N={} V={}  cycles={}",
            cpu.pc, f.zero as u8, f.carry as u8, f.negative as u8, f.overflow as u8, cpu.cycles);
    }
}

impl Default for Monitor {
    fn default() -> Self { Self::new() }
}

fn parse_u32(s: &str) -> Option<u32> {
    let s = s.trim();
    if let Some(h) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u32::from_str_radix(h, 16).ok()
    } else {
        s.parse().ok()
    }
}
