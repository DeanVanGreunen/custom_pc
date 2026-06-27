mod ast;
mod codegen;
mod error;
mod instruction;
mod lexer;
mod parser;
mod symbol_table;

use std::{fs, path::PathBuf, process};
use codegen::assemble;
use error::AsmResult;
use lexer::lex;
use parser::Parser;

fn run() -> AsmResult<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: asm <input.asm> [output.bin]");
        process::exit(1);
    }

    let input_path = PathBuf::from(&args[1]);
    let output_path = if args.len() >= 3 {
        PathBuf::from(&args[2])
    } else {
        input_path.with_extension("bin")
    };

    let src = fs::read_to_string(&input_path)
        .map_err(|e| error::AsmError::new(0, format!("cannot read '{}': {e}", input_path.display())))?;

    let tokens = lex(&src)?;
    let mut parser = Parser::new(tokens);
    let items = parser.parse()?;
    let assembled = assemble(&items)?;

    fs::write(&output_path, &assembled.image)
        .map_err(|e| error::AsmError::new(0, format!("cannot write '{}': {e}", output_path.display())))?;

    // Print symbol map to stderr so stdout can be piped.
    let mut syms: Vec<_> = assembled.symbols.iter().collect();
    syms.sort_by_key(|(name, _)| *name);
    for (name, addr) in syms {
        eprintln!("{addr:04X}  {name}");
    }

    eprintln!(
        "assembled {} bytes → {}",
        assembled.image.len(),
        output_path.display()
    );
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        process::exit(1);
    }
}
