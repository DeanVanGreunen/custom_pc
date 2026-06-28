use std::collections::HashSet;

/// A set of breakpoint addresses.
#[derive(Default)]
pub struct Breakpoints {
    addrs: HashSet<u32>,
}

impl Breakpoints {
    pub fn new() -> Self { Self::default() }

    pub fn add(&mut self, addr: u32) { self.addrs.insert(addr); }

    pub fn remove(&mut self, addr: u32) { self.addrs.remove(&addr); }

    pub fn clear(&mut self) { self.addrs.clear(); }

    pub fn is_set(&self, addr: u32) -> bool { self.addrs.contains(&addr) }

    pub fn list(&self) -> Vec<u32> {
        let mut v: Vec<_> = self.addrs.iter().copied().collect();
        v.sort();
        v
    }
}
