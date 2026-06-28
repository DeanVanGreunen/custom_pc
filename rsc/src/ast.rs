#[derive(Debug, Clone, PartialEq)]
pub enum Type { U32, I32, Bool }

#[derive(Debug, Clone, PartialEq)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    BitAnd, BitOr, BitXor,
    And, Or,
    Eq, Ne, Lt, Gt, Le, Ge,
}

#[derive(Debug, Clone)]
pub enum UnOp { Neg, Not }

#[derive(Debug, Clone)]
pub enum Expr {
    Lit(u32),
    Var(String),
    BinOp(Box<Expr>, BinOp, Box<Expr>),
    UnOp(UnOp, Box<Expr>),
    Call(String, Vec<Expr>),
    Assign(String, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let { name: String, _mutable: bool, init: Option<Expr> },
    Expr(Expr),
    If { cond: Expr, then_body: Vec<Stmt>, else_body: Vec<Stmt> },
    While { cond: Expr, body: Vec<Stmt> },
    Loop { body: Vec<Stmt> },
    Return(Option<Expr>),
}

#[derive(Debug, Clone)]
pub struct Param { pub name: String, pub ty: Type }

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub ret_type: Option<Type>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub struct Program { pub functions: Vec<Function> }
