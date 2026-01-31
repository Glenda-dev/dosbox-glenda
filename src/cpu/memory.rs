use alloc::vec;
use alloc::vec::Vec;

pub struct Memory {
    pub data: Vec<u8>, // 1MB Real Mode Memory
}

impl Memory {
    pub fn new() -> Self {
        Self { data: vec![0; 640 * 1024] }
    }

    pub fn read_u8(&self, addr: usize) -> u8 {
        if addr < self.data.len() { self.data[addr] } else { 0xFF }
    }

    pub fn write_u8(&mut self, addr: usize, val: u8) {
        if addr < self.data.len() {
            self.data[addr] = val;
        }
    }

    pub fn read_u16(&self, addr: usize) -> u16 {
        let low = self.read_u8(addr) as u16;
        let high = self.read_u8(addr + 1) as u16;
        (high << 8) | low
    }

    pub fn write_u16(&mut self, addr: usize, val: u16) {
        self.write_u8(addr, (val & 0xFF) as u8);
        self.write_u8(addr + 1, (val >> 8) as u8);
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}
