//! Memory-mapped I/O bus.
//!
//! I/O port map:
//!   0x0000 – write: emit byte to stdout  / read: always 0
//!   0x0001 – read:  read byte from stdin / write: ignored
//!   Other ports currently unimplemented.

use std::io::{Read, Write};
use crate::error::{EmuResult, EmulatorError};

pub struct Bus;

impl Bus {
    pub fn new() -> Self { Bus }

    pub fn io_read(&mut self, port: u16) -> EmuResult<u16> {
        match port {
            0x0000 => Ok(0),
            0x0001 => {
                let mut buf = [0u8; 1];
                match std::io::stdin().read(&mut buf) {
                    Ok(0) => Ok(0xFFFF), // EOF
                    Ok(_) => Ok(buf[0] as u16),
                    Err(_) => Ok(0xFFFF),
                }
            }
            other => Err(EmulatorError::IllegalIoPort(other)),
        }
    }

    pub fn io_write(&mut self, port: u16, value: u16) -> EmuResult<()> {
        match port {
            0x0000 => {
                let byte = (value & 0xFF) as u8;
                std::io::stdout().write_all(&[byte]).ok();
                Ok(())
            }
            0x0001 => Ok(()), // stdin write is a no-op
            other => Err(EmulatorError::IllegalIoPort(other)),
        }
    }
}

impl Default for Bus {
    fn default() -> Self { Self::new() }
}
