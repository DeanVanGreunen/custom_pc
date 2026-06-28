mod ast;
mod codegen;
mod error;
mod lexer;
mod parser;

use std::{env, fs, process};
use codegen::Codegen;
use lexer::Lexer;
use parser::Parser;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: rsc <source.rs> <output.bin>");
        process::exit(1);
    }

    let src_path = &args[1];
    let out_path = &args[2];

    let source = fs::read_to_string(src_path).unwrap_or_else(|e| {
        eprintln!("error: cannot read {src_path}: {e}");
        process::exit(1);
    });

    let tokens = Lexer::new(&source).tokenize().unwrap_or_else(|e| {
        eprintln!("error: {e}");
        process::exit(1);
    });

    let program = Parser::new(tokens).parse_program().unwrap_or_else(|e| {
        eprintln!("error: {e}");
        process::exit(1);
    });

    let binary = Codegen::new().compile(&program).unwrap_or_else(|e| {
        eprintln!("error: {e}");
        process::exit(1);
    });

    fs::write(out_path, &binary).unwrap_or_else(|e| {
        eprintln!("error: cannot write {out_path}: {e}");
        process::exit(1);
    });

    eprintln!("compiled {} bytes → {out_path}", binary.len());
}
