//! Abstract syntax tree nodes produced by the parser.

/// A single top-level item in an assembly source file.
#[derive(Debug, Clone)]
pub enum Item {
    /// A label definition: `name:`
    Label(String),
    /// `.equ NAME, value` — symbolic constant.
    Equ { name: String, value: i64, line: usize },
    /// `.org address` — set the current assembly address.
    Org { addr: u32, line: usize },
    /// An assembly instruction.
    Instruction { mnemonic: String, operands: Vec<Operand>, line: usize },
    /// A data directive (`.byte`, `.word`, `.string`).
    Data { kind: DataKind, args: Vec<DataArg>, line: usize },
}

/// An instruction operand.
#[derive(Debug, Clone)]
pub enum Operand {
    /// A general-purpose register: `r0`–`r15`.
    Reg(u8),
    /// Memory-indirect register: `[r0]`–`[r15]`.
    Mem(u8),
    /// Numeric immediate value.
    Imm(i64),
    /// Symbolic reference (label or `.equ` constant).
    Symbol(String),
    /// IEEE 754 float immediate (used with `fldi`).
    FImm(f64),
}

/// Selects which data directive produced a [`Item::Data`] node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataKind {
    Byte,
    Word,
    Float,
}

/// One argument to a data directive.
#[derive(Debug, Clone)]
pub enum DataArg {
    Number(i64),
    Symbol(String),
    Str(String),
    Float(f64),
}
