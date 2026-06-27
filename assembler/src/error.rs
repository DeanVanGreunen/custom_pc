use std::fmt;

#[derive(Debug)]
pub struct AsmError {
    pub line: usize,
    pub message: String,
}

impl AsmError {
    pub fn new(line: usize, message: impl Into<String>) -> Self {
        AsmError { line, message: message.into() }
    }
}

impl fmt::Display for AsmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(f, "error: {}", self.message)
        } else {
            write!(f, "error at line {}: {}", self.line, self.message)
        }
    }
}

impl std::error::Error for AsmError {}

pub type AsmResult<T> = Result<T, AsmError>;
