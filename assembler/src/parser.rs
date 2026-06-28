//! Recursive-descent parser: token stream → Vec<Item>.

use crate::ast::{DataArg, DataKind, Item, Operand};
use crate::error::{AsmError, AsmResult};
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    /// Line counter (Newline tokens advance this).
    line: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0, line: 1 }
    }

    pub fn parse(&mut self) -> AsmResult<Vec<Item>> {
        let mut items = Vec::new();
        while !self.at_end() {
            self.parse_line(&mut items)?;
        }
        Ok(items)
    }

    // ── helpers ──────────────────────────────────────────────────────────────

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        if tok == Some(&Token::Newline) { self.line += 1; }
        self.pos += 1;
        tok
    }

    fn eat_newlines(&mut self) {
        while self.peek() == Some(&Token::Newline) {
            self.advance();
        }
    }

    fn expect_newline(&mut self) -> AsmResult<()> {
        match self.peek() {
            Some(Token::Newline) | None => { self.advance(); Ok(()) }
            Some(t) => Err(AsmError::new(self.line, format!("expected newline, found {t:?}"))),
        }
    }

    fn expect_comma(&mut self) -> AsmResult<()> {
        match self.peek() {
            Some(Token::Comma) => { self.advance(); Ok(()) }
            other => Err(AsmError::new(self.line, format!("expected ',', found {other:?}"))),
        }
    }

    // ── parse_line ────────────────────────────────────────────────────────────

    fn parse_line(&mut self, items: &mut Vec<Item>) -> AsmResult<()> {
        self.eat_newlines();
        if self.at_end() { return Ok(()); }

        // A line can start with: IDENT (label or mnemonic), DOT (directive), or NEWLINE.
        loop {
            match self.peek() {
                Some(Token::Ident(_)) => {
                    // Could be `label:` or a mnemonic.
                    let name = match self.advance() {
                        Some(Token::Ident(s)) => s.clone(),
                        _ => unreachable!(),
                    };
                    if self.peek() == Some(&Token::Colon) {
                        self.advance(); // consume ':'
                        items.push(Item::Label(name));
                        // After a label the same line may have an instruction.
                        continue;
                    } else {
                        // It's an instruction mnemonic.
                        let line = self.line;
                        let operands = self.parse_operands()?;
                        items.push(Item::Instruction { mnemonic: name.to_lowercase(), operands, line });
                        break;
                    }
                }
                Some(Token::Dot) => {
                    self.advance();
                    let line = self.line;
                    self.parse_directive(items, line)?;
                    break;
                }
                Some(Token::Newline) | None => break,
                Some(t) => {
                    return Err(AsmError::new(self.line, format!("unexpected token {t:?}")));
                }
            }
        }

        self.expect_newline()
    }

    // ── directives ────────────────────────────────────────────────────────────

    fn parse_directive(&mut self, items: &mut Vec<Item>, line: usize) -> AsmResult<()> {
        let name = match self.advance() {
            Some(Token::Ident(s)) => s.to_lowercase(),
            other => return Err(AsmError::new(line, format!("expected directive name, found {other:?}"))),
        };

        match name.as_str() {
            "byte" => {
                let args = self.parse_data_args()?;
                items.push(Item::Data { kind: DataKind::Byte, args, line });
            }
            "word" => {
                let args = self.parse_data_args()?;
                items.push(Item::Data { kind: DataKind::Word, args, line });
            }
            "string" => {
                // Treat like .byte but accepts strings directly.
                let args = self.parse_data_args()?;
                items.push(Item::Data { kind: DataKind::Byte, args, line });
            }
            "float" => {
                let args = self.parse_data_args()?;
                items.push(Item::Data { kind: DataKind::Float, args, line });
            }
            "org" => {
                let addr = self.parse_imm_u32()?;
                items.push(Item::Org { addr, line });
            }
            "equ" => {
                let sym_name = match self.advance() {
                    Some(Token::Ident(s)) => s.clone(),
                    other => return Err(AsmError::new(line, format!("expected identifier after .equ, found {other:?}"))),
                };
                // Optional comma between name and value.
                if self.peek() == Some(&Token::Comma) { self.advance(); }
                let value = self.parse_imm_i64()?;
                items.push(Item::Equ { name: sym_name, value, line });
            }
            other => return Err(AsmError::new(line, format!("unknown directive '.{other}'"))),
        }
        Ok(())
    }

    // ── operands ──────────────────────────────────────────────────────────────

    fn parse_operands(&mut self) -> AsmResult<Vec<Operand>> {
        let mut ops = Vec::new();
        // If the next token is a newline or EOF there are no operands.
        if matches!(self.peek(), Some(Token::Newline) | None) {
            return Ok(ops);
        }
        ops.push(self.parse_operand()?);
        while self.peek() == Some(&Token::Comma) {
            self.advance();
            ops.push(self.parse_operand()?);
        }
        Ok(ops)
    }

    fn parse_operand(&mut self) -> AsmResult<Operand> {
        let line = self.line;
        match self.peek() {
            Some(Token::Reg(_)) => {
                let r = match self.advance() { Some(Token::Reg(r)) => *r, _ => unreachable!() };
                Ok(Operand::Reg(r))
            }
            Some(Token::LBracket) => {
                self.advance();
                let r = match self.advance() {
                    Some(Token::Reg(r)) => *r,
                    other => return Err(AsmError::new(line, format!("expected register inside '[', found {other:?}"))),
                };
                match self.advance() {
                    Some(Token::RBracket) => {}
                    other => return Err(AsmError::new(line, format!("expected ']', found {other:?}"))),
                }
                Ok(Operand::Mem(r))
            }
            Some(Token::Number(_)) => {
                let n = match self.advance() { Some(Token::Number(n)) => *n, _ => unreachable!() };
                Ok(Operand::Imm(n))
            }
            Some(Token::Float(_)) => {
                let f = match self.advance() { Some(Token::Float(f)) => *f, _ => unreachable!() };
                Ok(Operand::FImm(f))
            }
            Some(Token::Ident(_)) => {
                let s = match self.advance() { Some(Token::Ident(s)) => s.clone(), _ => unreachable!() };
                Ok(Operand::Symbol(s))
            }
            other => Err(AsmError::new(line, format!("expected operand, found {other:?}"))),
        }
    }

    // ── data args ─────────────────────────────────────────────────────────────

    fn parse_data_args(&mut self) -> AsmResult<Vec<DataArg>> {
        let mut args = Vec::new();
        if matches!(self.peek(), Some(Token::Newline) | None) {
            return Ok(args);
        }
        args.push(self.parse_data_arg()?);
        while self.peek() == Some(&Token::Comma) {
            self.advance();
            args.push(self.parse_data_arg()?);
        }
        Ok(args)
    }

    fn parse_data_arg(&mut self) -> AsmResult<DataArg> {
        let line = self.line;
        match self.peek() {
            Some(Token::Number(_)) => {
                let n = match self.advance() { Some(Token::Number(n)) => *n, _ => unreachable!() };
                Ok(DataArg::Number(n))
            }
            Some(Token::Float(_)) => {
                let f = match self.advance() { Some(Token::Float(f)) => *f, _ => unreachable!() };
                Ok(DataArg::Float(f))
            }
            Some(Token::Str(_)) => {
                let s = match self.advance() { Some(Token::Str(s)) => s.clone(), _ => unreachable!() };
                Ok(DataArg::Str(s))
            }
            Some(Token::Ident(_)) => {
                let s = match self.advance() { Some(Token::Ident(s)) => s.clone(), _ => unreachable!() };
                Ok(DataArg::Symbol(s))
            }
            other => Err(AsmError::new(line, format!("expected data argument, found {other:?}"))),
        }
    }

    // ── immediate helpers ─────────────────────────────────────────────────────

    fn parse_imm_i64(&mut self) -> AsmResult<i64> {
        let line = self.line;
        match self.advance() {
            Some(Token::Number(n)) => Ok(*n),
            other => Err(AsmError::new(line, format!("expected integer, found {other:?}"))),
        }
    }

    fn parse_imm_u32(&mut self) -> AsmResult<u32> {
        let line = self.line;
        let n = self.parse_imm_i64()?;
        if n < 0 || n > 0xFFFF_FFFF {
            return Err(AsmError::new(line, format!("address 0x{n:X} out of 32-bit range")));
        }
        Ok(n as u32)
    }
}
