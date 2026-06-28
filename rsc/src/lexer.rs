use crate::error::{CompileError, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Fn, Let, Mut, If, Else, While, Loop, Return, Break, Continue,
    U32, I32, Bool,
    True, False,
    IntLit(u32),
    Ident(String),
    // two-char ops
    AmpAmp, PipePipe, EqEq, BangEq, LtEq, GtEq, Arrow,
    // single-char ops
    Plus, Minus, Star, Slash, Percent,
    Amp, Pipe, Caret, Bang,
    Eq, Lt, Gt,
    // delimiters
    LParen, RParen, LBrace, RBrace, Comma, Semi, Colon,
    Eof,
}

pub struct Lexer { chars: Vec<char>, pos: usize, pub line: usize }

impl Lexer {
    pub fn new(src: &str) -> Self {
        Lexer { chars: src.chars().collect(), pos: 0, line: 1 }
    }

    fn peek(&self) -> Option<char> { self.chars.get(self.pos).copied() }
    fn peek2(&self) -> Option<char> { self.chars.get(self.pos + 1).copied() }
    fn advance(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied();
        if let Some(ch) = c { if ch == '\n' { self.line += 1; } self.pos += 1; }
        c
    }

    pub fn tokenize(&mut self) -> Result<Vec<(Token, usize)>> {
        let mut out = Vec::new();
        loop {
            self.skip_ws();
            let line = self.line;
            let Some(c) = self.peek() else {
                out.push((Token::Eof, line));
                break;
            };
            let tok = match c {
                '0'..='9' => self.lex_int()?,
                'a'..='z' | 'A'..='Z' | '_' => self.lex_word(),
                '+' => { self.advance(); Token::Plus }
                '-' => { self.advance(); if self.peek() == Some('>') { self.advance(); Token::Arrow } else { Token::Minus } }
                '*' => { self.advance(); Token::Star }
                '/' => { self.advance(); Token::Slash }
                '%' => { self.advance(); Token::Percent }
                '&' => { self.advance(); if self.peek() == Some('&') { self.advance(); Token::AmpAmp } else { Token::Amp } }
                '|' => { self.advance(); if self.peek() == Some('|') { self.advance(); Token::PipePipe } else { Token::Pipe } }
                '^' => { self.advance(); Token::Caret }
                '!' => { self.advance(); if self.peek() == Some('=') { self.advance(); Token::BangEq } else { Token::Bang } }
                '=' => { self.advance(); if self.peek() == Some('=') { self.advance(); Token::EqEq } else { Token::Eq } }
                '<' => { self.advance(); if self.peek() == Some('=') { self.advance(); Token::LtEq } else { Token::Lt } }
                '>' => { self.advance(); if self.peek() == Some('=') { self.advance(); Token::GtEq } else { Token::Gt } }
                '(' => { self.advance(); Token::LParen }
                ')' => { self.advance(); Token::RParen }
                '{' => { self.advance(); Token::LBrace }
                '}' => { self.advance(); Token::RBrace }
                ',' => { self.advance(); Token::Comma }
                ';' => { self.advance(); Token::Semi }
                ':' => { self.advance(); Token::Colon }
                ch => return Err(CompileError::Lex { line, msg: format!("unexpected char '{ch}'") }),
            };
            out.push((tok, line));
        }
        Ok(out)
    }

    fn skip_ws(&mut self) {
        loop {
            while self.peek().map(|c| c.is_whitespace()).unwrap_or(false) { self.advance(); }
            if self.peek() == Some('/') && self.peek2() == Some('/') {
                while self.peek().map(|c| c != '\n').unwrap_or(false) { self.advance(); }
            } else { break; }
        }
    }

    fn lex_int(&mut self) -> Result<Token> {
        let line = self.line;
        let mut s = String::new();
        if self.peek() == Some('0') && (self.peek2() == Some('x') || self.peek2() == Some('X')) {
            self.advance(); self.advance(); // consume 0x
            while self.peek().map(|c| c.is_ascii_hexdigit() || c == '_').unwrap_or(false) {
                let ch = self.advance().unwrap();
                if ch != '_' { s.push(ch); }
            }
            u32::from_str_radix(&s, 16).map(Token::IntLit)
                .map_err(|_| CompileError::Lex { line, msg: format!("bad hex literal 0x{s}") })
        } else {
            while self.peek().map(|c| c.is_ascii_digit() || c == '_').unwrap_or(false) {
                let ch = self.advance().unwrap();
                if ch != '_' { s.push(ch); }
            }
            s.parse::<u32>().map(Token::IntLit)
                .map_err(|_| CompileError::Lex { line, msg: format!("integer out of range: {s}") })
        }
    }

    fn lex_word(&mut self) -> Token {
        let mut s = String::new();
        while self.peek().map(|c| c.is_alphanumeric() || c == '_').unwrap_or(false) {
            s.push(self.advance().unwrap());
        }
        match s.as_str() {
            "fn"       => Token::Fn,
            "let"      => Token::Let,
            "mut"      => Token::Mut,
            "if"       => Token::If,
            "else"     => Token::Else,
            "while"    => Token::While,
            "loop"     => Token::Loop,
            "return"   => Token::Return,
            "break"    => Token::Break,
            "continue" => Token::Continue,
            "u32"      => Token::U32,
            "i32"      => Token::I32,
            "bool"     => Token::Bool,
            "true"     => Token::True,
            "false"    => Token::False,
            _          => Token::Ident(s),
        }
    }
}
