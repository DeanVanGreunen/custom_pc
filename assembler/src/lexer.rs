//! Tokeniser for the assembly language.

use crate::error::{AsmError, AsmResult};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Reg(u8),
    Number(i64),
    Str(String),
    Dot,
    Colon,
    Comma,
    LBracket,
    RBracket,
    Newline,
}

pub fn lex(src: &str) -> AsmResult<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = src.char_indices().peekable();
    let mut line = 1usize;

    while let Some((i, ch)) = chars.next() {
        match ch {
            // Skip whitespace (but not newlines)
            ' ' | '\t' | '\r' => {}

            '\n' => {
                if tokens.last() != Some(&Token::Newline) {
                    tokens.push(Token::Newline);
                }
                line += 1;
            }

            // Comment: skip to end of line
            ';' => {
                while let Some(&(_, c)) = chars.peek() {
                    if c == '\n' { break; }
                    chars.next();
                }
            }

            ',' => tokens.push(Token::Comma),
            ':' => tokens.push(Token::Colon),
            '[' => tokens.push(Token::LBracket),
            ']' => tokens.push(Token::RBracket),
            '.' => tokens.push(Token::Dot),

            '"' => {
                let mut s = String::new();
                loop {
                    match chars.next() {
                        None => return Err(AsmError::new(line, "unterminated string literal")),
                        Some((_, '"')) => break,
                        Some((_, '\\')) => match chars.next() {
                            Some((_, 'n'))  => s.push('\n'),
                            Some((_, 't'))  => s.push('\t'),
                            Some((_, 'r'))  => s.push('\r'),
                            Some((_, '0'))  => s.push('\0'),
                            Some((_, '"'))  => s.push('"'),
                            Some((_, '\\')) => s.push('\\'),
                            Some((_, c)) => return Err(AsmError::new(line, format!("unknown escape '\\{c}'"))),
                            None => return Err(AsmError::new(line, "unterminated string literal")),
                        },
                        Some((_, c)) => s.push(c),
                    }
                }
                tokens.push(Token::Str(s));
            }

            // Hex / binary / decimal numbers
            '0' if matches!(chars.peek(), Some((_, 'x' | 'X'))) => {
                chars.next(); // consume 'x'
                let mut hex = String::new();
                while let Some(&(_, c)) = chars.peek() {
                    if c.is_ascii_hexdigit() { hex.push(c); chars.next(); }
                    else if c == '_' { chars.next(); }
                    else { break; }
                }
                if hex.is_empty() {
                    return Err(AsmError::new(line, "expected hex digits after '0x'"));
                }
                let n = i64::from_str_radix(&hex, 16)
                    .map_err(|_| AsmError::new(line, format!("invalid hex literal '0x{hex}'")))?;
                tokens.push(Token::Number(n));
            }

            '0' if matches!(chars.peek(), Some((_, 'b' | 'B'))) => {
                chars.next();
                let mut bin = String::new();
                while let Some(&(_, c)) = chars.peek() {
                    if c == '0' || c == '1' { bin.push(c); chars.next(); }
                    else if c == '_' { chars.next(); }
                    else { break; }
                }
                if bin.is_empty() {
                    return Err(AsmError::new(line, "expected binary digits after '0b'"));
                }
                let n = i64::from_str_radix(&bin, 2)
                    .map_err(|_| AsmError::new(line, "invalid binary literal"))?;
                tokens.push(Token::Number(n));
            }

            c if c.is_ascii_digit() || (c == '-' && chars.peek().map_or(false, |(_, d)| d.is_ascii_digit())) => {
                let mut num = String::new();
                if c == '-' { num.push('-'); } else { num.push(c); }
                while let Some(&(_, d)) = chars.peek() {
                    if d.is_ascii_digit() || d == '_' {
                        if d != '_' { num.push(d); }
                        chars.next();
                    } else {
                        break;
                    }
                }
                let n = num.parse::<i64>()
                    .map_err(|_| AsmError::new(line, format!("invalid integer '{num}'")))?;
                tokens.push(Token::Number(n));
            }

            c if c.is_alphabetic() || c == '_' => {
                let mut ident = String::new();
                ident.push(c);
                while let Some(&(_, d)) = chars.peek() {
                    if d.is_alphanumeric() || d == '_' {
                        ident.push(d);
                        chars.next();
                    } else {
                        break;
                    }
                }
                // Detect register names r0–r15
                if let Some(reg) = parse_reg(&ident) {
                    tokens.push(Token::Reg(reg));
                } else {
                    tokens.push(Token::Ident(ident));
                }
            }

            other => {
                let _ = i;
                return Err(AsmError::new(line, format!("unexpected character '{other}'")));
            }
        }
    }

    // Ensure file ends with a newline token so the parser always sees a terminator.
    if tokens.last() != Some(&Token::Newline) {
        tokens.push(Token::Newline);
    }

    Ok(tokens)
}

fn parse_reg(s: &str) -> Option<u8> {
    let rest = s.strip_prefix('r').or_else(|| s.strip_prefix('R'))?;
    let n: u8 = rest.parse().ok()?;
    if n <= 15 { Some(n) } else { None }
}
