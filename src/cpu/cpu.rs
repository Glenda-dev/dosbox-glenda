use crate::cpu::bios;
use crate::cpu::instructions::Opcode;
use crate::cpu::memory::Memory;
use crate::cpu::regs::Registers;

pub struct Cpu {
    pub regs: Registers,
    pub memory: Memory,
    pub disk_image: alloc::vec::Vec<u8>,
    pub seg_override: Option<u16>,
}

impl Cpu {
    pub fn new(rom: &[u8]) -> Self {
        let mut cpu = Self {
            regs: Registers::default(),
            memory: Memory::new(),
            disk_image: alloc::vec::Vec::from(rom),
            seg_override: None,
        };
        // Load ONLY the boot sector (first 512 bytes) at 0000:7C00
        let boot_sector = if rom.len() >= 512 { &rom[0..512] } else { rom };
        cpu.load_rom(boot_sector, 0x7C00);
        cpu.regs.cs = 0x0000;
        cpu.regs.ip = 0x7C00;
        cpu.regs.sp = 0x7C00; // Standard for many boot sectors
        cpu.regs.dx = 0x0000; // Boot from drive A:

        // Initialize IVT for INT 1Eh (Disk Parameter Table) at 0000:0078
        // Point it to 0000:0EF0 (arbitrary safe location in low memory)
        cpu.memory.write_u16(0x0078, 0x0EF0);
        cpu.memory.write_u16(0x007A, 0x0000);

        // Floppy Disk Parameter Table (DPT)
        // 720K typical: AF 02 25 02 09 1B FF 54 F6 01 08
        let dpt = [0xAF, 0x02, 0x25, 0x02, 0x09, 0x1B, 0xFF, 0x54, 0xF6, 0x01, 0x08];
        for (i, &b) in dpt.iter().enumerate() {
            cpu.memory.write_u8(0x0EF0 + i, b);
        }

        cpu
    }

    pub fn load_rom(&mut self, rom: &[u8], addr: usize) {
        for (i, &byte) in rom.iter().enumerate() {
            if addr + i < self.memory.data.len() {
                self.memory.data[addr + i] = byte;
            }
        }
    }

    pub fn update_flags_isz(&mut self, val: u8) {
        // Zero Flag (Bit 6)
        if val == 0 {
            self.regs.flags |= 0x0040;
        } else {
            self.regs.flags &= !0x0040;
        }

        // Sign Flag (Bit 7)
        if (val & 0x80) != 0 {
            self.regs.flags |= 0x0080;
        } else {
            self.regs.flags &= !0x0080;
        }
    }

    pub fn fetch_u8(&mut self) -> u8 {
        let pc = (self.regs.cs as usize * 16) + self.regs.ip as usize;
        let val = self.memory.read_u8(pc);
        self.regs.ip = self.regs.ip.wrapping_add(1);
        val
    }

    pub fn fetch_u16(&mut self) -> u16 {
        let low = self.fetch_u8() as u16;
        let high = self.fetch_u8() as u16;
        (high << 8) | low
    }

    pub fn get_reg16(&self, reg: u8) -> u16 {
        match reg {
            0 => self.regs.ax,
            1 => self.regs.cx,
            2 => self.regs.dx,
            3 => self.regs.bx,
            4 => self.regs.sp,
            5 => self.regs.bp,
            6 => self.regs.si,
            7 => self.regs.di,
            _ => 0,
        }
    }

    pub fn set_reg16(&mut self, reg: u8, val: u16) {
        match reg {
            0 => self.regs.ax = val,
            1 => self.regs.cx = val,
            2 => self.regs.dx = val,
            3 => self.regs.bx = val,
            4 => self.regs.sp = val,
            5 => self.regs.bp = val,
            6 => self.regs.si = val,
            7 => self.regs.di = val,
            _ => {}
        }
    }

    pub fn get_reg8(&self, reg: u8) -> u8 {
        match reg {
            0 => (self.regs.ax & 0xFF) as u8,
            1 => (self.regs.cx & 0xFF) as u8,
            2 => (self.regs.dx & 0xFF) as u8,
            3 => (self.regs.bx & 0xFF) as u8,
            4 => (self.regs.ax >> 8) as u8,
            5 => (self.regs.cx >> 8) as u8,
            6 => (self.regs.dx >> 8) as u8,
            7 => (self.regs.bx >> 8) as u8,
            _ => 0,
        }
    }

    pub fn set_reg8(&mut self, reg: u8, val: u8) {
        match reg {
            0 => self.regs.ax = (self.regs.ax & 0xFF00) | val as u16,
            1 => self.regs.cx = (self.regs.cx & 0xFF00) | val as u16,
            2 => self.regs.dx = (self.regs.dx & 0xFF00) | val as u16,
            3 => self.regs.bx = (self.regs.bx & 0xFF00) | val as u16,
            4 => self.regs.ax = (self.regs.ax & 0x00FF) | ((val as u16) << 8),
            5 => self.regs.cx = (self.regs.cx & 0x00FF) | ((val as u16) << 8),
            6 => self.regs.dx = (self.regs.dx & 0x00FF) | ((val as u16) << 8),
            7 => self.regs.bx = (self.regs.bx & 0x00FF) | ((val as u16) << 8),
            _ => {}
        }
    }

    pub fn push16(&mut self, val: u16) {
        self.regs.sp = self.regs.sp.wrapping_sub(2);
        let addr = (self.regs.ss as usize * 16) + self.regs.sp as usize;
        self.memory.write_u16(addr, val);
    }

    pub fn pop16(&mut self) -> u16 {
        let addr = (self.regs.ss as usize * 16) + self.regs.sp as usize;
        let val = self.memory.read_u16(addr);
        self.regs.sp = self.regs.sp.wrapping_add(2);
        val
    }

    pub fn step(&mut self) -> bool {
        let opcode = Opcode::decode(self);
        let ret = opcode.execute(self);
        self.seg_override = None;
        ret
    }

    pub fn handle_interrupt(&mut self, int_no: u8) -> bool {
        bios::handle_interrupt(self, int_no)
    }

    pub fn dump_registers(&self) {
        glenda::println!("Registers:");
        glenda::println!(
            "  AX: {:04x}  BX: {:04x}  CX: {:04x}  DX: {:04x}",
            self.regs.ax, self.regs.bx, self.regs.cx, self.regs.dx
        );
        glenda::println!(
            "  SI: {:04x}  DI: {:04x}  SP: {:04x}  BP: {:04x}",
            self.regs.si, self.regs.di, self.regs.sp, self.regs.bp
        );
        glenda::println!(
            "  CS: {:04x}  DS: {:04x}  ES: {:04x}  SS: {:04x}",
            self.regs.cs, self.regs.ds, self.regs.es, self.regs.ss
        );
        glenda::println!("  IP: {:04x}  Flags: {:04x}", self.regs.ip, self.regs.flags);
    }
}
