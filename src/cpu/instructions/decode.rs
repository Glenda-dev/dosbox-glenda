use crate::cpu::cpu::Cpu;
use crate::cpu::instructions::Opcode;

pub fn decode(cpu: &mut Cpu) -> Opcode {
    let opcode = cpu.fetch_u8();
    match opcode {
        0x26 => {
            cpu.seg_override = Some(cpu.regs.es);
            Opcode::decode(cpu)
        }
        0x2E => {
            cpu.seg_override = Some(cpu.regs.cs);
            Opcode::decode(cpu)
        }
        0x36 => {
            cpu.seg_override = Some(cpu.regs.ss);
            Opcode::decode(cpu)
        }
        0x3E => {
            cpu.seg_override = Some(cpu.regs.ds);
            Opcode::decode(cpu)
        }
        0x90 => Opcode::Nop,
        0xF4 => Opcode::Hlt,
        0xFA => Opcode::Cli,
        0xFB => Opcode::Sti,
        0xFC => Opcode::Cld,
        0xFD => Opcode::Std,
        0xF8 => Opcode::Clc,
        0xF9 => Opcode::Stc,
        0xF5 => Opcode::Cmc,
        0x98 => Opcode::Cbw,
        0x99 => Opcode::Cwd,
        0xE8 => {
            let rel = cpu.fetch_u16() as i16;
            Opcode::CallNear(rel)
        }
        0xC3 => Opcode::Ret,
        0xCA => Opcode::RetFarImm(cpu.fetch_u16()),
        0x91..=0x97 => Opcode::XchgAxReg(opcode - 0x90),
        0xE9 => {
            let rel = cpu.fetch_u16() as i16;
            Opcode::JmpRel16(rel)
        }
        0xEA => {
            let ip = cpu.fetch_u16();
            let cs = cpu.fetch_u16();
            Opcode::JmpFar(ip, cs)
        }
        0xEB => {
            let rel = cpu.fetch_u8() as i8;
            Opcode::JmpShort(rel)
        }
        0x30 => Opcode::XorRm8Reg8(cpu.fetch_u8()),
        0x31 => Opcode::XorRm16Reg16(cpu.fetch_u8()),
        0x32 => Opcode::XorReg8Rm8(cpu.fetch_u8()),
        0x33 => Opcode::XorReg16Rm16(cpu.fetch_u8()),
        0xC4 => Opcode::Les(cpu.fetch_u8()),
        0xC5 => Opcode::Lds(cpu.fetch_u8()),
        0x8D => Opcode::Lea(cpu.fetch_u8()),
        0x40..=0x47 => Opcode::IncReg16(opcode - 0x40),
        0x48..=0x4F => Opcode::DecReg16(opcode - 0x48),
        0x50..=0x57 => Opcode::PushReg16(opcode - 0x50),
        0x58..=0x5F => Opcode::PopReg16(opcode - 0x58),
        0x06 | 0x0E | 0x16 | 0x1E => Opcode::PushSreg(opcode),
        0x07 | 0x17 | 0x1F => Opcode::PopSreg(opcode),
        0x8E => Opcode::MovSregRm(cpu.fetch_u8()),
        0x8C => Opcode::MovRmSreg(cpu.fetch_u8()),
        0x8F => Opcode::PopRm16(cpu.fetch_u8()),
        0x88 => Opcode::MovRmReg8(cpu.fetch_u8()),
        0x89 => Opcode::MovRmReg16(cpu.fetch_u8()),
        0x8A => Opcode::MovReg8Rm8(cpu.fetch_u8()),
        0x8B => Opcode::MovReg16Rm16(cpu.fetch_u8()),
        0x86 => Opcode::XchgRegRm8(cpu.fetch_u8()),
        0x87 => Opcode::XchgRegRm16(cpu.fetch_u8()),
        0xB0..=0xB7 => {
            let val = cpu.fetch_u8();
            Opcode::MovReg8Imm8(opcode - 0xB0, val)
        }
        0xB8..=0xBF => {
            let val = cpu.fetch_u16();
            Opcode::MovReg16Imm16(opcode - 0xB8, val)
        }
        0xC6 => {
            let modrm = cpu.fetch_u8();
            let val = cpu.fetch_u8();
            Opcode::MovRm8Imm8(modrm, val)
        }
        0xC7 => {
            let modrm = cpu.fetch_u8();
            let val = cpu.fetch_u16();
            Opcode::MovRm16Imm16(modrm, val)
        }
        0xCD => Opcode::Int(cpu.fetch_u8()),
        0xC8 => {
            let size = cpu.fetch_u16();
            let level = cpu.fetch_u8();
            Opcode::Enter(size, level)
        }
        0xC9 => Opcode::Leave,
        0xE0 => Opcode::Loopne(cpu.fetch_u8() as i8),
        0xE1 => Opcode::Loope(cpu.fetch_u8() as i8),
        0xE3 => Opcode::JcxzShort(cpu.fetch_u8() as i8),
        0xE2 => Opcode::Loop(cpu.fetch_u8() as i8),
        0x70..=0x7F => {
            let rel = cpu.fetch_u8() as i8;
            Opcode::JccShort(opcode, rel)
        }
        0xF3 => Opcode::Rep(alloc::boxed::Box::new(Opcode::decode(cpu))),
        0xA0 => Opcode::MovAccMoff8(cpu.fetch_u16()),
        0xA1 => Opcode::MovAccMoff16(cpu.fetch_u16()),
        0xA2 => Opcode::MovMoff8Acc(cpu.fetch_u16()),
        0xA3 => Opcode::MovMoff16Acc(cpu.fetch_u16()),
        0xA4 => Opcode::Movsb,
        0xA5 => Opcode::Movsw,
        0xA6 => Opcode::Cmpsb,
        0xA7 => Opcode::Cmpsw,
        0xAA => Opcode::Stosb,
        0xAB => Opcode::Stosw,
        0xAC => Opcode::Lodsb,
        0xE4 => Opcode::InAlImm8(cpu.fetch_u8()),
        0xE5 => Opcode::InAxImm8(cpu.fetch_u8()),
        0xE6 => Opcode::OutImm8Al(cpu.fetch_u8()),
        0xE7 => Opcode::OutImm8Ax(cpu.fetch_u8()),
        0xEC => Opcode::InAlDx,
        0xED => Opcode::InAxDx,
        0xEE => Opcode::OutDxAl,
        0xEF => Opcode::OutDxAx,
        0x00 => Opcode::AddRm8Reg8(cpu.fetch_u8()),
        0x01 => Opcode::AddRm16Reg16(cpu.fetch_u8()),
        0x02 => Opcode::AddReg8Rm8(cpu.fetch_u8()),
        0x03 => Opcode::AddReg16Rm16(cpu.fetch_u8()),
        0x04 => Opcode::AddAccImm8(cpu.fetch_u8()),
        0x05 => Opcode::AddAccImm16(cpu.fetch_u16()),
        0x08 => Opcode::OrRm8Reg8(cpu.fetch_u8()),
        0x09 => Opcode::OrRm16Reg16(cpu.fetch_u8()),
        0x0A => Opcode::OrReg8Rm8(cpu.fetch_u8()),
        0x0B => Opcode::OrReg16Rm16(cpu.fetch_u8()),
        0x0C => Opcode::OrAccImm8(cpu.fetch_u8()),
        0x0D => Opcode::OrAccImm16(cpu.fetch_u16()),
        0x10 => Opcode::AdcRm8Reg8(cpu.fetch_u8()),
        0x11 => Opcode::AdcRm16Reg16(cpu.fetch_u8()),
        0x12 => Opcode::AdcReg8Rm8(cpu.fetch_u8()),
        0x13 => Opcode::AdcReg16Rm16(cpu.fetch_u8()),
        0x14 => Opcode::AdcAccImm8(cpu.fetch_u8()),
        0x15 => Opcode::AdcAccImm16(cpu.fetch_u16()),
        0x18 => Opcode::SbbRm8Reg8(cpu.fetch_u8()),
        0x19 => Opcode::SbbRm16Reg16(cpu.fetch_u8()),
        0x1A => Opcode::SbbReg8Rm8(cpu.fetch_u8()),
        0x1B => Opcode::SbbReg16Rm16(cpu.fetch_u8()),
        0x1C => Opcode::SbbAccImm8(cpu.fetch_u8()),
        0x1D => Opcode::SbbAccImm16(cpu.fetch_u16()),
        0x20 => Opcode::AndRm8Reg8(cpu.fetch_u8()),
        0x21 => Opcode::AndRm16Reg16(cpu.fetch_u8()),
        0x22 => Opcode::AndReg8Rm8(cpu.fetch_u8()),
        0x23 => Opcode::AndReg16Rm16(cpu.fetch_u8()),
        0x24 => Opcode::AndAccImm8(cpu.fetch_u8()),
        0x25 => Opcode::AndAccImm16(cpu.fetch_u16()),
        0x28 => Opcode::SubRm8Reg8(cpu.fetch_u8()),
        0x29 => Opcode::SubRm16Reg16(cpu.fetch_u8()),
        0x2A => Opcode::SubReg8Rm8(cpu.fetch_u8()),
        0x2B => Opcode::SubReg16Rm16(cpu.fetch_u8()),
        0x2C => Opcode::SubAccImm8(cpu.fetch_u8()),
        0x2D => Opcode::SubAccImm16(cpu.fetch_u16()),
        0x34 => Opcode::XorAccImm8(cpu.fetch_u8()),
        0x35 => Opcode::XorAccImm16(cpu.fetch_u16()),
        0x38 => Opcode::CmpRm8Reg8(cpu.fetch_u8()),
        0x39 => Opcode::CmpRm16Reg16(cpu.fetch_u8()),
        0x3A => Opcode::CmpReg8Rm8(cpu.fetch_u8()),
        0x3B => Opcode::CmpReg16Rm16(cpu.fetch_u8()),
        0x3C => Opcode::CmpAccImm8(cpu.fetch_u8()),
        0x3D => Opcode::CmpAccImm16(cpu.fetch_u16()),
        0x80 | 0x81 | 0x83 => Opcode::Grp1(opcode, cpu.fetch_u8()),
        0xC0 | 0xC1 => {
            let modrm = cpu.fetch_u8();
            let count = cpu.fetch_u8();
            Opcode::Grp2Imm8(opcode, modrm, count)
        }
        0xD0..=0xD3 => Opcode::Grp2(opcode, cpu.fetch_u8()),
        0xF6 | 0xF7 => Opcode::Grp3(opcode, cpu.fetch_u8()),
        0x9C => Opcode::Pushf,
        0x9D => Opcode::Popf,
        0x9E => Opcode::Sahf,
        0x9F => Opcode::Lahf,
        0x84 => Opcode::TestRm8Reg8(cpu.fetch_u8()),
        0x85 => Opcode::TestRm16Reg16(cpu.fetch_u8()),
        _ => Opcode::Unknown(opcode),
    }
}
