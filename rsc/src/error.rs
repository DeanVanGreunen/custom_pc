use std::fmt;

#[derive(Debug)]
pub enum CompileError {
    Lex   { line: usize, msg: String },
    Parse { line: usize, msg: String },
    Codegen(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lex   { line, msg } => write!(f, "lex error at line {line}: {msg}"),
            Self::Parse { line, msg } => write!(f, "parse error at line {line}: {msg}"),
            Self::Codegen(msg)        => write!(f, "codegen error: {msg}"),
        }
    }
}

pub type Result<T> = std::result::Result<T, CompileError>;
