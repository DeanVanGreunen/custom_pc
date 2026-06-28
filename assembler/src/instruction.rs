//! Opcode table for the custom CPU ISA.

/// Pack two 4-bit register indices into one byte: high nibble = rd, low nibble = rs.
#[inline]
pub fn pack_regs(rd: u8, rs: u8) -> u8 {
    (rd << 4) | (rs & 0x0F)
}

/// Operand shape — determines how many bytes an instruction encodes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperandFormat {
    /// No operands. Instruction is 1 byte.
    None,
    /// One register. Instruction is 2 bytes (opcode + reg byte).
    Reg,
    /// Two registers. Instruction is 2 bytes (opcode + packed reg byte).
    RegReg,
    /// Register + 16-bit immediate. Instruction is 4 bytes.
    RegImm,
    /// 16-bit immediate only. Instruction is 3 bytes.
    Imm,
}

macro_rules! opcodes {
    ($( $variant:ident = $byte:expr, $mnemonic:literal, $fmt:ident );* $(;)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Opcode {
            $( $variant, )*
        }

        impl Opcode {
            pub fn from_mnemonic(m: &str) -> Option<Self> {
                match m {
                    $( $mnemonic => Some(Self::$variant), )*
                    _ => None,
                }
            }

            pub fn as_u8(self) -> u8 {
                match self {
                    $( Self::$variant => $byte, )*
                }
            }

            pub fn mnemonic(self) -> &'static str {
                match self {
                    $( Self::$variant => $mnemonic, )*
                }
            }

            pub fn format(self) -> OperandFormat {
                match self {
                    $( Self::$variant => OperandFormat::$fmt, )*
                }
            }

            pub fn length(self) -> usize {
                match self.format() {
                    OperandFormat::None   => 1,
                    OperandFormat::Reg    => 2,
                    OperandFormat::RegReg => 2,
                    OperandFormat::RegImm => 6,
                    OperandFormat::Imm    => 5,
                }
            }
        }
    };
}

opcodes! {
    Nop  = 0x00, "nop",  None;
    Hlt  = 0x01, "hlt",  None;
    Mov  = 0x02, "mov",  RegReg;
    Ldi  = 0x03, "ldi",  RegImm;
    Ld   = 0x04, "ld",   RegReg;
    St   = 0x05, "st",   RegReg;
    Ldb  = 0x06, "ldb",  RegReg;
    Stb  = 0x07, "stb",  RegReg;
    Add  = 0x08, "add",  RegReg;
    Sub  = 0x09, "sub",  RegReg;
    And  = 0x0A, "and",  RegReg;
    Or   = 0x0B, "or",   RegReg;
    Xor  = 0x0C, "xor",  RegReg;
    Not  = 0x0D, "not",  Reg;
    Shl  = 0x0E, "shl",  RegReg;
    Shr  = 0x0F, "shr",  RegReg;
    Addi = 0x10, "addi", RegImm;
    Subi = 0x11, "subi", RegImm;
    Andi = 0x12, "andi", RegImm;
    Ori  = 0x13, "ori",  RegImm;
    Xori = 0x14, "xori", RegImm;
    Mul  = 0x17, "mul",  RegReg;
    Div  = 0x18, "div",  RegReg;
    Mod  = 0x19, "mod",  RegReg;
    Cmp  = 0x15, "cmp",  RegReg;
    Cmpi = 0x16, "cmpi", RegImm;
    Push = 0x20, "push", Reg;
    Pop  = 0x21, "pop",  Reg;
    Call = 0x22, "call", Imm;
    Ret  = 0x23, "ret",  None;
    Jmp  = 0x30, "jmp",  Imm;
    Jz   = 0x31, "jz",   Imm;
    Jnz  = 0x32, "jnz",  Imm;
    Jc   = 0x33, "jc",   Imm;
    Jnc  = 0x34, "jnc",  Imm;
    Jn   = 0x35, "jn",   Imm;
    Jnn  = 0x36, "jnn",  Imm;
    Jv   = 0x37, "jv",   Imm;
    Jnv  = 0x38, "jnv",  Imm;
    Jgt  = 0x39, "jgt",  Imm;
    Jlt  = 0x3A, "jlt",  Imm;
    Jge  = 0x3B, "jge",  Imm;
    Jle  = 0x3C, "jle",  Imm;
    In   = 0x40, "in",   RegImm;
    Out  = 0x41, "out",  RegImm;
    Fadd  = 0x50, "fadd",  RegReg;
    Fsub  = 0x51, "fsub",  RegReg;
    Fmul  = 0x52, "fmul",  RegReg;
    Fdiv  = 0x53, "fdiv",  RegReg;
    Fmod  = 0x54, "fmod",  RegReg;
    Fneg  = 0x55, "fneg",  Reg;
    Fabs  = 0x56, "fabs",  Reg;
    Fsqrt = 0x57, "fsqrt", Reg;
    Fcmp  = 0x58, "fcmp",  RegReg;
    Ftoi  = 0x59, "ftoi",  Reg;
    Itof  = 0x5A, "itof",  Reg;
    Fldi  = 0x5F, "fldi",  RegImm;
}
