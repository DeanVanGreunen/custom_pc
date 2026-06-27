use std::fmt;

#[derive(Debug)]
pub enum EmulatorError {
    InvalidOpcode(u8),
    IllegalMemoryAccess(u16),
    IllegalIoPort(u16),
    StackOverflow,
    StackUnderflow,
    Halted,
}

impl fmt::Display for EmulatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOpcode(op)         => write!(f, "invalid opcode 0x{op:02X}"),
            Self::IllegalMemoryAccess(addr) => write!(f, "illegal memory access at 0x{addr:04X}"),
            Self::IllegalIoPort(port)       => write!(f, "unknown I/O port 0x{port:04X}"),
            Self::StackOverflow             => write!(f, "stack overflow"),
            Self::StackUnderflow            => write!(f, "stack underflow"),
            Self::Halted                    => write!(f, "CPU halted"),
        }
    }
}

impl std::error::Error for EmulatorError {}

pub type EmuResult<T> = Result<T, EmulatorError>;
