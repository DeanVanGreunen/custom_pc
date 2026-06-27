use std::collections::HashSet;

/// A set of breakpoint addresses.
#[derive(Default)]
pub struct Breakpoints {
    addrs: HashSet<u16>,
}

impl Breakpoints {
    pub fn new() -> Self { Self::default() }

    pub fn add(&mut self, addr: u16) { self.addrs.insert(addr); }

    pub fn remove(&mut self, addr: u16) { self.addrs.remove(&addr); }

    pub fn clear(&mut self) { self.addrs.clear(); }

    pub fn is_set(&self, addr: u16) -> bool { self.addrs.contains(&addr) }

    pub fn list(&self) -> Vec<u16> {
        let mut v: Vec<_> = self.addrs.iter().copied().collect();
        v.sort();
        v
    }
}
