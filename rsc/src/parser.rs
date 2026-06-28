use crate::ast::*;
use crate::error::{CompileError, Result};
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<(Token, usize)>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<(Token, usize)>) -> Self { Parser { tokens, pos: 0 } }

    fn peek(&self) -> &Token { &self.tokens[self.pos].0 }
    fn line(&self) -> usize { self.tokens[self.pos].1 }

    fn eat(&mut self) -> Token {
        let tok = self.tokens[self.pos].0.clone();
        if self.pos + 1 < self.tokens.len() { self.pos += 1; }
        tok
    }

    fn expect(&mut self, want: &Token) -> Result<()> {
        if self.peek() == want { self.eat(); Ok(()) }
        else { Err(CompileError::Parse { line: self.line(), msg: format!("expected {want:?}, got {:?}", self.peek()) }) }
    }

    fn eat_ident(&mut self) -> Result<String> {
        match self.eat() {
            Token::Ident(s) => Ok(s),
            tok => Err(CompileError::Parse { line: self.line(), msg: format!("expected identifier, got {tok:?}") }),
        }
    }

    fn eat_type(&mut self) -> Result<Type> {
        match self.eat() {
            Token::U32  => Ok(Type::U32),
            Token::I32  => Ok(Type::I32),
            Token::Bool => Ok(Type::Bool),
            tok => Err(CompileError::Parse { line: self.line(), msg: format!("expected type, got {tok:?}") }),
        }
    }

    pub fn parse_program(&mut self) -> Result<Program> {
        let mut fns = Vec::new();
        while self.peek() != &Token::Eof { fns.push(self.parse_fn()?); }
        Ok(Program { functions: fns })
    }

    fn parse_fn(&mut self) -> Result<Function> {
        self.expect(&Token::Fn)?;
        let name = self.eat_ident()?;
        self.expect(&Token::LParen)?;
        let mut params = Vec::new();
        while self.peek() != &Token::RParen {
            if self.peek() == &Token::Mut { self.eat(); } // allow mut on params
            let pname = self.eat_ident()?;
            self.expect(&Token::Colon)?;
            let ty = self.eat_type()?;
            params.push(Param { name: pname, ty });
            if self.peek() == &Token::Comma { self.eat(); }
        }
        self.expect(&Token::RParen)?;
        let ret_type = if self.peek() == &Token::Arrow { self.eat(); Some(self.eat_type()?) } else { None };
        self.expect(&Token::LBrace)?;
        let body = self.parse_stmts()?;
        self.expect(&Token::RBrace)?;
        Ok(Function { name, params, ret_type, body })
    }

    fn parse_stmts(&mut self) -> Result<Vec<Stmt>> {
        let mut stmts = Vec::new();
        while self.peek() != &Token::RBrace && self.peek() != &Token::Eof {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt> {
        match self.peek().clone() {
            Token::Let => {
                self.eat();
                let mutable = if self.peek() == &Token::Mut { self.eat(); true } else { false };
                let name = self.eat_ident()?;
                if self.peek() == &Token::Colon { self.eat(); self.eat_type()?; }
                let init = if self.peek() == &Token::Eq { self.eat(); Some(self.parse_expr()?) } else { None };
                self.expect(&Token::Semi)?;
                Ok(Stmt::Let { name, _mutable: mutable, init })
            }
            Token::If => {
                self.eat();
                let cond = self.parse_expr()?;
                self.expect(&Token::LBrace)?;
                let then_body = self.parse_stmts()?;
                self.expect(&Token::RBrace)?;
                let else_body = if self.peek() == &Token::Else {
                    self.eat();
                    self.expect(&Token::LBrace)?;
                    let b = self.parse_stmts()?;
                    self.expect(&Token::RBrace)?;
                    b
                } else { vec![] };
                Ok(Stmt::If { cond, then_body, else_body })
            }
            Token::While => {
                self.eat();
                let cond = self.parse_expr()?;
                self.expect(&Token::LBrace)?;
                let body = self.parse_stmts()?;
                self.expect(&Token::RBrace)?;
                Ok(Stmt::While { cond, body })
            }
            Token::Loop => {
                self.eat();
                self.expect(&Token::LBrace)?;
                let body = self.parse_stmts()?;
                self.expect(&Token::RBrace)?;
                Ok(Stmt::Loop { body })
            }
            Token::Return => {
                self.eat();
                let val = if self.peek() != &Token::Semi { Some(self.parse_expr()?) } else { None };
                self.expect(&Token::Semi)?;
                Ok(Stmt::Return(val))
            }
            _ => {
                let e = self.parse_expr()?;
                self.expect(&Token::Semi)?;
                Ok(Stmt::Expr(e))
            }
        }
    }

    fn parse_expr(&mut self) -> Result<Expr> { self.parse_assign() }

    fn parse_assign(&mut self) -> Result<Expr> {
        let lhs = self.parse_or()?;
        if self.peek() == &Token::Eq {
            self.eat();
            let rhs = self.parse_assign()?;
            if let Expr::Var(name) = lhs {
                return Ok(Expr::Assign(name, Box::new(rhs)));
            }
            return Err(CompileError::Parse { line: self.line(), msg: "lhs of = must be a variable".into() });
        }
        Ok(lhs)
    }

    fn parse_or(&mut self) -> Result<Expr> {
        let mut e = self.parse_and()?;
        while self.peek() == &Token::PipePipe { self.eat(); e = Expr::BinOp(Box::new(e), BinOp::Or, Box::new(self.parse_and()?)); }
        Ok(e)
    }

    fn parse_and(&mut self) -> Result<Expr> {
        let mut e = self.parse_cmp()?;
        while self.peek() == &Token::AmpAmp { self.eat(); e = Expr::BinOp(Box::new(e), BinOp::And, Box::new(self.parse_cmp()?)); }
        Ok(e)
    }

    fn parse_cmp(&mut self) -> Result<Expr> {
        let mut e = self.parse_bitor()?;
        loop {
            let op = match self.peek() {
                Token::EqEq => BinOp::Eq,  Token::BangEq => BinOp::Ne,
                Token::Lt   => BinOp::Lt,  Token::Gt     => BinOp::Gt,
                Token::LtEq => BinOp::Le,  Token::GtEq   => BinOp::Ge,
                _ => break,
            };
            self.eat();
            e = Expr::BinOp(Box::new(e), op, Box::new(self.parse_bitor()?));
        }
        Ok(e)
    }

    fn parse_bitor(&mut self) -> Result<Expr> {
        let mut e = self.parse_bitxor()?;
        while self.peek() == &Token::Pipe { self.eat(); e = Expr::BinOp(Box::new(e), BinOp::BitOr, Box::new(self.parse_bitxor()?)); }
        Ok(e)
    }

    fn parse_bitxor(&mut self) -> Result<Expr> {
        let mut e = self.parse_bitand()?;
        while self.peek() == &Token::Caret { self.eat(); e = Expr::BinOp(Box::new(e), BinOp::BitXor, Box::new(self.parse_bitand()?)); }
        Ok(e)
    }

    fn parse_bitand(&mut self) -> Result<Expr> {
        let mut e = self.parse_add()?;
        while self.peek() == &Token::Amp { self.eat(); e = Expr::BinOp(Box::new(e), BinOp::BitAnd, Box::new(self.parse_add()?)); }
        Ok(e)
    }

    fn parse_add(&mut self) -> Result<Expr> {
        let mut e = self.parse_mul()?;
        loop {
            let op = match self.peek() { Token::Plus => BinOp::Add, Token::Minus => BinOp::Sub, _ => break };
            self.eat();
            e = Expr::BinOp(Box::new(e), op, Box::new(self.parse_mul()?));
        }
        Ok(e)
    }

    fn parse_mul(&mut self) -> Result<Expr> {
        let mut e = self.parse_unary()?;
        loop {
            let op = match self.peek() { Token::Star => BinOp::Mul, Token::Slash => BinOp::Div, Token::Percent => BinOp::Mod, _ => break };
            self.eat();
            e = Expr::BinOp(Box::new(e), op, Box::new(self.parse_unary()?));
        }
        Ok(e)
    }

    fn parse_unary(&mut self) -> Result<Expr> {
        match self.peek() {
            Token::Minus => { self.eat(); Ok(Expr::UnOp(UnOp::Neg, Box::new(self.parse_unary()?))) }
            Token::Bang  => { self.eat(); Ok(Expr::UnOp(UnOp::Not, Box::new(self.parse_unary()?))) }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr> {
        match self.eat() {
            Token::IntLit(n) => Ok(Expr::Lit(n)),
            Token::True      => Ok(Expr::Lit(1)),
            Token::False     => Ok(Expr::Lit(0)),
            Token::Ident(name) => {
                if self.peek() == &Token::LParen {
                    self.eat();
                    let mut args = Vec::new();
                    while self.peek() != &Token::RParen {
                        args.push(self.parse_expr()?);
                        if self.peek() == &Token::Comma { self.eat(); }
                    }
                    self.expect(&Token::RParen)?;
                    Ok(Expr::Call(name, args))
                } else {
                    Ok(Expr::Var(name))
                }
            }
            Token::LParen => { let e = self.parse_expr()?; self.expect(&Token::RParen)?; Ok(e) }
            tok => Err(CompileError::Parse { line: self.line(), msg: format!("unexpected token {tok:?}") }),
        }
    }
}
