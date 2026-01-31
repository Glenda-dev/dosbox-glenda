use crate::cpu::cpu::Cpu;

impl Cpu {
    pub fn get_offset(&mut self, modrm: u8) -> (u16, u16) {
        let mode = (modrm >> 6) & 0x03;
        let rm = modrm & 0x07;
        let mut disp: i16 = 0;

        if mode == 1 {
            disp = self.fetch_u8() as i8 as i16;
        } else if mode == 2 {
            disp = self.fetch_u16() as i16;
        }

        let base = match rm {
            0 => self.regs.bx.wrapping_add(self.regs.si),
            1 => self.regs.bx.wrapping_add(self.regs.di),
            2 => self.regs.bp.wrapping_add(self.regs.si),
            3 => self.regs.bp.wrapping_add(self.regs.di),
            4 => self.regs.si,
            5 => self.regs.di,
            6 => {
                if mode == 0 {
                    let off = self.fetch_u16();
                    return (off, self.regs.ds);
                } else {
                    self.regs.bp
                }
            }
            7 => self.regs.bx,
            _ => 0,
        };

        let offset = base.wrapping_add(disp as u16);
        let seg =
            if rm == 2 || rm == 3 || (rm == 6 && mode != 0) { self.regs.ss } else { self.regs.ds };
        (offset, seg)
    }

    pub fn get_ea(&mut self, modrm: u8) -> usize {
        let (offset, mut seg) = self.get_offset(modrm);
        if let Some(s) = self.seg_override {
            seg = s;
        }
        (seg as usize * 16) + offset as usize
    }

    pub fn get_seg(&self, default: u16) -> u16 {
        self.seg_override.unwrap_or(default)
    }

    pub fn set_rm16(&mut self, modrm: u8, val: u16) {
        if (modrm & 0xC0) == 0xC0 {
            self.set_reg16(modrm & 0x07, val);
        } else {
            let addr = self.get_ea(modrm);
            self.memory.write_u16(addr, val);
        }
    }

    pub fn get_rm16(&mut self, modrm: u8) -> u16 {
        if (modrm & 0xC0) == 0xC0 {
            self.get_reg16(modrm & 0x07)
        } else {
            let addr = self.get_ea(modrm);
            self.memory.read_u16(addr)
        }
    }

    pub fn set_rm8(&mut self, modrm: u8, val: u8) {
        if (modrm & 0xC0) == 0xC0 {
            self.set_reg8(modrm & 0x07, val);
        } else {
            let addr = self.get_ea(modrm);
            self.memory.write_u8(addr, val);
        }
    }

    pub fn get_rm8(&mut self, modrm: u8) -> u8 {
        if (modrm & 0xC0) == 0xC0 {
            self.get_reg8(modrm & 0x07)
        } else {
            let addr = self.get_ea(modrm);
            self.memory.read_u8(addr)
        }
    }

    pub fn read_mem8(&self, seg: u16, off: u16) -> u8 {
        let addr = (seg as usize * 16) + off as usize;
        self.memory.read_u8(addr)
    }

    pub fn read_mem16(&self, seg: u16, off: u16) -> u16 {
        let addr = (seg as usize * 16) + off as usize;
        self.memory.read_u16(addr)
    }

    pub fn write_mem8(&mut self, seg: u16, off: u16, val: u8) {
        let addr = (seg as usize * 16) + off as usize;
        self.memory.write_u8(addr, val);
    }

    pub fn write_mem16(&mut self, seg: u16, off: u16, val: u16) {
        let addr = (seg as usize * 16) + off as usize;
        self.memory.write_u16(addr, val);
    }

    pub fn update_flags_isz16(&mut self, val: u16) {
        if val == 0 {
            self.regs.flags |= 0x0040;
        } else {
            self.regs.flags &= !0x0040;
        }
        if (val & 0x8000) != 0 {
            self.regs.flags |= 0x0080;
        } else {
            self.regs.flags &= !0x0080;
        }
    }
}
