//! Serial UART device.
//!
//! I/O port 0x20 – TX: write a byte to the transmit buffer
//! I/O port 0x21 – RX: read a byte from the receive buffer (0xFF = empty)
//! I/O port 0x22 – STATUS: bit 0 = rx available, bit 1 = tx ready

use std::collections::VecDeque;

pub const PORT_TX:     u16 = 0x20;
pub const PORT_RX:     u16 = 0x21;
pub const PORT_STATUS: u16 = 0x22;

#[derive(Default)]
pub struct Serial {
    rx_buf: VecDeque<u8>,
    tx_buf: VecDeque<u8>,
}

impl Serial {
    pub fn new() -> Self { Self::default() }

    /// Feed incoming bytes into the RX buffer (called by host).
    pub fn feed_rx(&mut self, data: &[u8]) {
        self.rx_buf.extend(data.iter().copied());
    }

    /// Drain all bytes from the TX buffer (called by host).
    pub fn drain_tx(&mut self) -> Vec<u8> {
        self.tx_buf.drain(..).collect()
    }

    pub fn io_read(&mut self, port: u16) -> Option<u16> {
        match port {
            PORT_RX     => Some(self.rx_buf.pop_front().unwrap_or(0xFF) as u16),
            PORT_STATUS => {
                let rx_ready = !self.rx_buf.is_empty() as u16;
                let tx_ready = 1u16; // always ready
                Some(rx_ready | (tx_ready << 1))
            }
            _ => None,
        }
    }

    pub fn io_write(&mut self, port: u16, value: u16) -> bool {
        if port == PORT_TX {
            self.tx_buf.push_back((value & 0xFF) as u8);
            true
        } else {
            false
        }
    }
}
