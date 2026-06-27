//! Code generation: turn parsed [`Item`]s into a binary image.
//!
//! Assembly is two-pass:
//! 1. **Pass one** walks the items assigning an address to every label and
//!    recording `.equ` constants, using only instruction/data *sizes* (which
//!    never depend on a symbol's value).
//! 2. **Pass two** walks the items again and emits bytes, resolving symbol
//!    references against the now-complete symbol table.
//!
//! Addresses map directly onto image offsets (the image is meant to be loaded
//! at `0x0000`). A `.org` pads forward with zeroes; jumping backwards is an
//! error.

use crate::ast::{DataArg, DataKind, Item, Operand};
use crate::error::{AsmError, AsmResult};
use crate::instruction::{pack_regs, Opcode, OperandFormat};
use crate::symbol_table::SymbolTable;

/// The result of assembling a source file.
pub struct Assembled {
    /// The binary image, starting at address 0.
    pub image: Vec<u8>,
    /// The symbol table (labels + constants), for an optional map file.
    pub symbols: SymbolTable,
}

/// Assemble a list of items into a binary image.
pub fn assemble(items: &[Item]) -> AsmResult<Assembled> {
    let symbols = pass_one(items)?;
    let image = pass_two(items, &symbols)?;
    Ok(Assembled { image, symbols })
}

/// Pass one: assign addresses to labels and collect constants.
fn pass_one(items: &[Item]) -> AsmResult<SymbolTable> {
    let mut symbols = SymbolTable::new();
    let mut addr: usize = 0;
    for item in items {
        match item {
            Item::Label(name) => symbols.define(name, addr as i64, 0)?,
            Item::Equ { name, value, line } => symbols.define(name, *value, *line)?,
            Item::Org { addr: a, line } => {
                if (*a as usize) < addr {
                    return Err(AsmError::new(
                        *line,
                        format!(".org 0x{a:04X} moves backwards from 0x{addr:04X}"),
                    ));
                }
                addr = *a as usize;
            }
            Item::Instruction { mnemonic, operands, line } => {
                addr += instruction_length(mnemonic, operands, *line)?;
            }
            Item::Data { kind, args, line } => {
                addr += data_length(*kind, args, *line)?;
            }
        }
        if addr > 0x1_0000 {
            return Err(AsmError::new(0, "program exceeds 64 KiB address space"));
        }
    }
    Ok(symbols)
}

/// Pass two: emit bytes.
fn pass_two(items: &[Item], symbols: &SymbolTable) -> AsmResult<Vec<u8>> {
    let mut image: Vec<u8> = Vec::new();
    for item in items {
        match item {
            Item::Label(_) | Item::Equ { .. } => {}
            Item::Org { addr, .. } => {
                if image.len() < *addr as usize {
                    image.resize(*addr as usize, 0);
                }
            }
            Item::Instruction { mnemonic, operands, line } => {
                encode_instruction(mnemonic, operands, symbols, *line, &mut image)?;
            }
            Item::Data { kind, args, line } => {
                encode_data(*kind, args, symbols, *line, &mut image)?;
            }
        }
    }
    Ok(image)
}

/// Compute the encoded length of an instruction (pass one).
fn instruction_length(mnemonic: &str, operands: &[Operand], line: usize) -> AsmResult<usize> {
    let opcode = lookup_opcode(mnemonic, line)?;
    check_arity(opcode, operands, line)?;
    Ok(opcode.length())
}

/// Compute the encoded length of a data directive (pass one).
fn data_length(kind: DataKind, args: &[DataArg], _line: usize) -> AsmResult<usize> {
    let mut len = 0;
    for arg in args {
        len += match (kind, arg) {
            (DataKind::Word, _) => 2,
            (_, DataArg::Str(s)) => s.len(),
            (_, _) => 1,
        };
    }
    Ok(len)
}

fn lookup_opcode(mnemonic: &str, line: usize) -> AsmResult<Opcode> {
    Opcode::from_mnemonic(mnemonic)
        .ok_or_else(|| AsmError::new(line, format!("unknown instruction '{mnemonic}'")))
}

/// Validate operand count and shapes against the opcode's format.
fn check_arity(opcode: Opcode, operands: &[Operand], line: usize) -> AsmResult<()> {
    let expected = match opcode.format() {
        OperandFormat::None => 0,
        OperandFormat::Reg => 1,
        OperandFormat::RegReg => 2,
        OperandFormat::RegImm => 2,
        OperandFormat::Imm => 1,
    };
    if operands.len() != expected {
        return Err(AsmError::new(
            line,
            format!(
                "'{}' expects {} operand(s), found {}",
                opcode.mnemonic(),
                expected,
                operands.len()
            ),
        ));
    }
    Ok(())
}

fn encode_instruction(
    mnemonic: &str,
    operands: &[Operand],
    symbols: &SymbolTable,
    line: usize,
    out: &mut Vec<u8>,
) -> AsmResult<()> {
    let opcode = lookup_opcode(mnemonic, line)?;
    check_arity(opcode, operands, line)?;
    out.push(opcode.as_u8());

    use Opcode::*;
    match opcode.format() {
        OperandFormat::None => {}
        OperandFormat::Reg => {
            let rd = reg_of(&operands[0], line)?;
            out.push(pack_regs(rd, 0));
        }
        OperandFormat::RegReg => {
            // Memory-style ops use a [reg] operand in one slot.
            let (rd, rs) = match opcode {
                Ld | Ldb => (reg_of(&operands[0], line)?, mem_of(&operands[1], line)?),
                St | Stb => (mem_of(&operands[0], line)?, reg_of(&operands[1], line)?),
                _ => (reg_of(&operands[0], line)?, reg_of(&operands[1], line)?),
            };
            out.push(pack_regs(rd, rs));
        }
        OperandFormat::RegImm => {
            let rd = reg_of(&operands[0], line)?;
            let imm = imm_of(&operands[1], symbols, line)?;
            out.push(pack_regs(rd, 0));
            out.push((imm & 0xFF) as u8);
            out.push((imm >> 8) as u8);
        }
        OperandFormat::Imm => {
            let imm = imm_of(&operands[0], symbols, line)?;
            out.push((imm & 0xFF) as u8);
            out.push((imm >> 8) as u8);
        }
    }
    Ok(())
}

fn encode_data(
    kind: DataKind,
    args: &[DataArg],
    symbols: &SymbolTable,
    line: usize,
    out: &mut Vec<u8>,
) -> AsmResult<()> {
    for arg in args {
        match (kind, arg) {
            (DataKind::Word, DataArg::Number(n)) => {
                let w = *n as u16;
                out.push((w & 0xFF) as u8);
                out.push((w >> 8) as u8);
            }
            (DataKind::Word, DataArg::Symbol(name)) => {
                let w = symbols.resolve(name, line)? as u16;
                out.push((w & 0xFF) as u8);
                out.push((w >> 8) as u8);
            }
            (DataKind::Word, DataArg::Str(_)) => {
                return Err(AsmError::new(line, ".word does not take a string"));
            }
            (_, DataArg::Number(n)) => out.push(*n as u8),
            (_, DataArg::Symbol(name)) => out.push(symbols.resolve(name, line)? as u8),
            (_, DataArg::Str(s)) => out.extend_from_slice(s.as_bytes()),
        }
    }
    Ok(())
}

fn reg_of(op: &Operand, line: usize) -> AsmResult<u8> {
    match op {
        Operand::Reg(r) => Ok(*r),
        other => Err(AsmError::new(line, format!("expected a register, found {other:?}"))),
    }
}

fn mem_of(op: &Operand, line: usize) -> AsmResult<u8> {
    match op {
        Operand::Mem(r) => Ok(*r),
        other => Err(AsmError::new(line, format!("expected [register], found {other:?}"))),
    }
}

fn imm_of(op: &Operand, symbols: &SymbolTable, line: usize) -> AsmResult<u16> {
    match op {
        Operand::Imm(n) => Ok(*n as u16),
        Operand::Symbol(name) => Ok(symbols.resolve(name, line)? as u16),
        other => Err(AsmError::new(line, format!("expected an immediate, found {other:?}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;
    use crate::parser::Parser;

    fn asm(src: &str) -> Vec<u8> {
        let items = Parser::new(lex(src).unwrap()).parse().unwrap();
        assemble(&items).unwrap().image
    }

    #[test]
    fn encodes_ldi_hlt() {
        let img = asm("ldi r0, 0x1234\nhlt");
        assert_eq!(img, vec![0x03, 0x00, 0x34, 0x12, 0x01]);
    }

    #[test]
    fn resolves_forward_label() {
        // jmp end ; nop ; end: hlt
        let img = asm("jmp end\nnop\nend: hlt");
        // jmp (0x30) to address 0x0004, then nop (0x00) at 3, hlt at 4.
        assert_eq!(&img[0..3], &[0x30, 0x04, 0x00]);
        assert_eq!(img[3], 0x00); // nop
        assert_eq!(img[4], 0x01); // hlt
    }

    #[test]
    fn memory_ops_encode_registers() {
        let img = asm("ld r1, [r2]\nst [r3], r4");
        assert_eq!(img, vec![0x04, 0x12, 0x05, 0x34]);
    }

    #[test]
    fn data_directives() {
        let img = asm(".byte 1, 2\n.word 0xBEEF\n.string \"AB\"");
        assert_eq!(img, vec![1, 2, 0xEF, 0xBE, b'A', b'B']);
    }

    #[test]
    fn org_pads_forward() {
        let img = asm("nop\n.org 0x04\nhlt");
        assert_eq!(img, vec![0x00, 0, 0, 0, 0x01]);
    }
}