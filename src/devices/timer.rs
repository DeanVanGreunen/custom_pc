//! Countdown timer device.
//!
//! I/O port 0x30 – COUNTER: current value (read) / reload value (write)
//! I/O port 0x31 – PERIOD:  reload period in ticks
//! I/O port 0x32 – CONTROL: bit 0 = enable, bit 1 = irq enable
//!
//! `tick()` should be called once per CPU cycle.  When the counter reaches
//! zero it reloads from PERIOD and sets the fired flag.

pub const PORT_COUNTER: u16 = 0x30;
pub const PORT_PERIOD:  u16 = 0x31;
pub const PORT_CONTROL: u16 = 0x32;

#[derive(Default)]
pub struct Timer {
    counter: u16,
    period:  u16,
    enabled: bool,
    irq_en:  bool,
    pub fired: bool,
}

impl Timer {
    pub fn new() -> Self { Self::default() }

    pub fn tick(&mut self) {
        if !self.enabled || self.period == 0 { return; }
        if self.counter == 0 {
            self.counter = self.period;
            self.fired   = true;
        } else {
            self.counter -= 1;
        }
    }

    pub fn io_read(&self, port: u16) -> Option<u16> {
        match port {
            PORT_COUNTER => Some(self.counter),
            PORT_PERIOD  => Some(self.period),
            PORT_CONTROL => Some(self.enabled as u16 | ((self.irq_en as u16) << 1)),
            _ => None,
        }
    }

    pub fn io_write(&mut self, port: u16, value: u16) -> bool {
        match port {
            PORT_COUNTER => { self.counter = value; true }
            PORT_PERIOD  => { self.period  = value; true }
            PORT_CONTROL => {
                self.enabled = value & 1 != 0;
                self.irq_en  = value & 2 != 0;
                true
            }
            _ => false,
        }
    }
}
