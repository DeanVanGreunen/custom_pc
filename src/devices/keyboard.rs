//! Keyboard device: FIFO queue of key codes, exposed via I/O ports.
//!
//! I/O port 0x10 – STATUS: bit 0 = key available
//! I/O port 0x11 – DATA:   read = dequeue key code (0 if none)

use std::collections::VecDeque;

pub const PORT_KB_STATUS: u16 = 0x10;
pub const PORT_KB_DATA:   u16 = 0x11;

#[derive(Default)]
pub struct Keyboard {
    queue: VecDeque<u8>,
}

impl Keyboard {
    pub fn new() -> Self { Self::default() }

    /// Push a key code into the device FIFO (called by the host/UI).
    pub fn push_key(&mut self, code: u8) {
        self.queue.push_back(code);
    }

    pub fn io_read(&mut self, port: u16) -> Option<u16> {
        match port {
            PORT_KB_STATUS => Some(if self.queue.is_empty() { 0 } else { 1 }),
            PORT_KB_DATA   => Some(self.queue.pop_front().unwrap_or(0) as u16),
            _ => None,
        }
    }
}
