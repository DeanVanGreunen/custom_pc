//! Simple block-device (disk) with 512-byte sectors.
//!
//! I/O port 0x40 – COMMAND:  0=read, 1=write, 2=seek
//! I/O port 0x41 – SECTOR_LO: low 16 bits of sector address
//! I/O port 0x42 – SECTOR_HI: high 16 bits of sector address
//! I/O port 0x43 – DATA:     read/write one byte from/to the sector buffer
//! I/O port 0x44 – STATUS:   bit 0 = ready, bit 1 = error

pub const PORT_CMD:       u16 = 0x40;
pub const PORT_SECTOR_LO: u16 = 0x41;
pub const PORT_SECTOR_HI: u16 = 0x42;
pub const PORT_DATA:      u16 = 0x43;
pub const PORT_STATUS:    u16 = 0x44;

pub const SECTOR_SIZE: usize = 512;

pub struct Disk {
    storage:    Vec<u8>,
    sector_lo:  u16,
    sector_hi:  u16,
    buf:        [u8; SECTOR_SIZE],
    buf_pos:    usize,
    error:      bool,
}

impl Disk {
    pub fn new(capacity_sectors: usize) -> Self {
        Disk {
            storage:   vec![0u8; capacity_sectors * SECTOR_SIZE],
            sector_lo: 0,
            sector_hi: 0,
            buf:       [0u8; SECTOR_SIZE],
            buf_pos:   0,
            error:     false,
        }
    }

    fn sector(&self) -> usize {
        ((self.sector_hi as usize) << 16) | self.sector_lo as usize
    }

    fn do_read(&mut self) {
        let base = self.sector() * SECTOR_SIZE;
        if base + SECTOR_SIZE <= self.storage.len() {
            self.buf.copy_from_slice(&self.storage[base..base + SECTOR_SIZE]);
            self.buf_pos = 0;
            self.error   = false;
        } else {
            self.error = true;
        }
    }

    fn do_write(&mut self) {
        let base = self.sector() * SECTOR_SIZE;
        if base + SECTOR_SIZE <= self.storage.len() {
            self.storage[base..base + SECTOR_SIZE].copy_from_slice(&self.buf);
            self.error = false;
        } else {
            self.error = true;
        }
    }

    pub fn io_read(&mut self, port: u16) -> Option<u16> {
        match port {
            PORT_DATA   => {
                let b = if self.buf_pos < SECTOR_SIZE { self.buf[self.buf_pos] } else { 0xFF };
                self.buf_pos = (self.buf_pos + 1).min(SECTOR_SIZE);
                Some(b as u16)
            }
            PORT_STATUS => Some(1 | ((self.error as u16) << 1)),
            PORT_SECTOR_LO => Some(self.sector_lo),
            PORT_SECTOR_HI => Some(self.sector_hi),
            _ => None,
        }
    }

    pub fn io_write(&mut self, port: u16, value: u16) -> bool {
        match port {
            PORT_CMD => {
                match value & 0xFF {
                    0 => self.do_read(),
                    1 => self.do_write(),
                    2 => { self.buf_pos = 0; }
                    _ => self.error = true,
                }
                true
            }
            PORT_SECTOR_LO => { self.sector_lo = value; true }
            PORT_SECTOR_HI => { self.sector_hi = value; true }
            PORT_DATA => {
                if self.buf_pos < SECTOR_SIZE {
                    self.buf[self.buf_pos] = (value & 0xFF) as u8;
                    self.buf_pos += 1;
                }
                true
            }
            _ => false,
        }
    }
}
