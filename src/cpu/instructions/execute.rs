use super::Opcode;
use crate::cpu::cpu::Cpu;

impl Opcode {
    pub fn execute(&self, cpu: &mut Cpu) -> bool {
        match *self {
            Opcode::Nop => {}
            Opcode::XchgAxReg(reg) => {
                let tmp = cpu.regs.ax;
                cpu.regs.ax = cpu.get_reg16(reg);
                cpu.set_reg16(reg, tmp);
            }
            Opcode::Hlt => return false,
            Opcode::Cli => cpu.regs.flags &= !0x0200,
            Opcode::Sti => cpu.regs.flags |= 0x0200,
            Opcode::Cld => cpu.regs.flags &= !0x0400,
            Opcode::Std => cpu.regs.flags |= 0x0400,
            Opcode::Clc => cpu.regs.flags &= !0x0001,
            Opcode::Stc => cpu.regs.flags |= 0x0001,
            Opcode::Cmc => cpu.regs.flags ^= 0x0001,
            Opcode::Cbw => {
                let al = (cpu.regs.ax & 0xFF) as i8;
                cpu.regs.ax = al as i16 as u16;
            }
            Opcode::Cwd => {
                let ax = cpu.regs.ax as i16;
                cpu.regs.dx = if ax < 0 { 0xFFFF } else { 0 };
            }
            Opcode::CallNear(rel) => {
                let next_ip = cpu.regs.ip;
                cpu.push16(next_ip);
                cpu.regs.ip = cpu.regs.ip.wrapping_add(rel as u16);
            }
            Opcode::Ret => cpu.regs.ip = cpu.pop16(),
            Opcode::RetFarImm(imm) => {
                cpu.regs.ip = cpu.pop16();
                cpu.regs.cs = cpu.pop16();
                cpu.regs.sp = cpu.regs.sp.wrapping_add(imm);
            }
            Opcode::JmpRel16(rel) => cpu.regs.ip = cpu.regs.ip.wrapping_add(rel as u16),
            Opcode::JmpFar(ip, cs) => {
                cpu.regs.ip = ip;
                cpu.regs.cs = cs;
            }
            Opcode::JmpShort(rel) => cpu.regs.ip = cpu.regs.ip.wrapping_add(rel as u16),
            Opcode::XorRm16Reg16(modrm) => {
                let val1 = cpu.get_rm16(modrm);
                let val2 = cpu.get_reg16((modrm >> 3) & 0x07);
                let result = val1 ^ val2;
                cpu.set_rm16(modrm, result);
                cpu.update_flags_isz16(result);
            }
            Opcode::XorReg16Rm16(modrm) => {
                let val1 = cpu.get_reg16((modrm >> 3) & 0x07);
                let val2 = cpu.get_rm16(modrm);
                let result = val1 ^ val2;
                cpu.set_reg16((modrm >> 3) & 0x07, result);
                cpu.update_flags_isz16(result);
            }
            Opcode::XorReg8Rm8(modrm) => {
                let val1 = cpu.get_reg8((modrm >> 3) & 0x07);
                let val2 = cpu.get_rm8(modrm);
                let result = val1 ^ val2;
                cpu.set_reg8((modrm >> 3) & 0x07, result);
                cpu.update_flags_isz(result);
            }
            Opcode::Lea(modrm) => {
                let reg = (modrm >> 3) & 0x07;
                let (offset, _) = cpu.get_offset(modrm);
                cpu.set_reg16(reg, offset);
            }
            Opcode::Les(modrm) => {
                let reg = (modrm >> 3) & 0x07;
                let addr = cpu.get_ea(modrm);
                let off = cpu.memory.read_u16(addr);
                let seg = cpu.memory.read_u16(addr + 2);
                cpu.set_reg16(reg, off);
                cpu.regs.es = seg;
            }
            Opcode::Lds(modrm) => {
                let reg = (modrm >> 3) & 0x07;
                let addr = cpu.get_ea(modrm);
                let off = cpu.memory.read_u16(addr);
                let seg = cpu.memory.read_u16(addr + 2);
                cpu.set_reg16(reg, off);
                cpu.regs.ds = seg;
            }
            Opcode::IncReg16(reg) => {
                let val = cpu.get_reg16(reg);
                let res = val.wrapping_add(1);
                cpu.set_reg16(reg, res);
                cpu.update_flags_isz16(res);
            }
            Opcode::DecReg16(reg) => {
                let val = cpu.get_reg16(reg);
                let res = val.wrapping_sub(1);
                cpu.set_reg16(reg, res);
                cpu.update_flags_isz16(res);
            }
            Opcode::PushReg16(reg) => {
                let val = cpu.get_reg16(reg);
                cpu.push16(val);
            }
            Opcode::PopReg16(reg) => {
                let val = cpu.pop16();
                cpu.set_reg16(reg, val);
            }
            Opcode::PopRm16(modrm) => {
                let val = cpu.pop16();
                cpu.set_rm16(modrm, val);
            }
            Opcode::PushSreg(opcode) => {
                let val = match opcode {
                    0x06 => cpu.regs.es,
                    0x0E => cpu.regs.cs,
                    0x16 => cpu.regs.ss,
                    0x1E => cpu.regs.ds,
                    _ => 0,
                };
                cpu.push16(val);
            }
            Opcode::PopSreg(opcode) => {
                let val = cpu.pop16();
                match opcode {
                    0x07 => cpu.regs.es = val,
                    0x17 => cpu.regs.ss = val,
                    0x1F => cpu.regs.ds = val,
                    _ => {}
                }
            }
            Opcode::MovSregRm(modrm) => {
                let sreg = (modrm >> 3) & 0x07;
                let val = cpu.get_rm16(modrm);
                match sreg {
                    0 => cpu.regs.es = val,
                    1 => cpu.regs.cs = val,
                    2 => cpu.regs.ss = val,
                    3 => cpu.regs.ds = val,
                    _ => {}
                }
            }
            Opcode::MovRmSreg(modrm) => {
                let sreg = (modrm >> 3) & 0x07;
                let val = match sreg {
                    0 => cpu.regs.es,
                    1 => cpu.regs.cs,
                    2 => cpu.regs.ss,
                    3 => cpu.regs.ds,
                    _ => 0,
                };
                cpu.set_rm16(modrm, val);
            }
            Opcode::MovRmReg8(modrm) => {
                let val = cpu.get_reg8((modrm >> 3) & 0x07);
                cpu.set_rm8(modrm, val);
            }
            Opcode::MovRmReg16(modrm) => {
                let val = cpu.get_reg16((modrm >> 3) & 0x07);
                cpu.set_rm16(modrm, val);
            }
            Opcode::MovReg8Rm8(modrm) => {
                let val = cpu.get_rm8(modrm);
                cpu.set_reg8((modrm >> 3) & 0x07, val);
            }
            Opcode::MovReg16Rm16(modrm) => {
                let val = cpu.get_rm16(modrm);
                cpu.set_reg16((modrm >> 3) & 0x07, val);
            }
            Opcode::XchgRegRm8(modrm) => {
                let val1 = cpu.get_reg8((modrm >> 3) & 0x07);
                let val2 = cpu.get_rm8(modrm);
                cpu.set_reg8((modrm >> 3) & 0x07, val2);
                cpu.set_rm8(modrm, val1);
            }
            Opcode::XchgRegRm16(modrm) => {
                let val1 = cpu.get_reg16((modrm >> 3) & 0x07);
                let val2 = cpu.get_rm16(modrm);
                cpu.set_reg16((modrm >> 3) & 0x07, val2);
                cpu.set_rm16(modrm, val1);
            }
            Opcode::MovReg16Imm16(reg, val) => cpu.set_reg16(reg, val),
            Opcode::MovAccMoff8(offset) => {
                let seg = cpu.get_seg(cpu.regs.ds);
                let val = cpu.read_mem8(seg, offset);
                cpu.set_reg8(0, val); // AL
            }
            Opcode::MovAccMoff16(offset) => {
                let seg = cpu.get_seg(cpu.regs.ds);
                let val = cpu.read_mem16(seg, offset);
                cpu.regs.ax = val;
            }
            Opcode::MovMoff8Acc(offset) => {
                let seg = cpu.get_seg(cpu.regs.ds);
                let val = cpu.get_reg8(0);
                cpu.write_mem8(seg, offset, val);
            }
            Opcode::MovMoff16Acc(offset) => {
                let seg = cpu.get_seg(cpu.regs.ds);
                cpu.write_mem16(seg, offset, cpu.regs.ax);
            }
            Opcode::MovReg8Imm8(reg, val) => cpu.set_reg8(reg, val),
            Opcode::MovRm8Imm8(modrm, val) => cpu.set_rm8(modrm, val),
            Opcode::MovRm16Imm16(modrm, val) => cpu.set_rm16(modrm, val),
            Opcode::Int(int_no) => {
                let ret = cpu.handle_interrupt(int_no);
                if ret == false {
                    return false;
                }
            }
            Opcode::Enter(size, level) => {
                cpu.push16(cpu.regs.bp);
                let frame_ptr = cpu.regs.sp;
                if level > 0 {
                    let mut temp_bp = cpu.regs.bp;
                    for _ in 1..level {
                        temp_bp = temp_bp.wrapping_sub(2);
                        let val = cpu.read_mem16(cpu.regs.ss, temp_bp);
                        cpu.push16(val);
                    }
                    cpu.push16(frame_ptr);
                }
                cpu.regs.bp = frame_ptr;
                cpu.regs.sp = cpu.regs.sp.wrapping_sub(size);
            }
            Opcode::Leave => {
                cpu.regs.sp = cpu.regs.bp;
                cpu.regs.bp = cpu.pop16();
            }
            Opcode::JccShort(opcode, rel) => {
                let condition = match opcode & 0x0F {
                    0x0 => (cpu.regs.flags & 0x0800) != 0, // JO
                    0x1 => (cpu.regs.flags & 0x0800) == 0, // JNO
                    0x2 => (cpu.regs.flags & 0x0001) != 0, // JB/JNAE/JC
                    0x3 => (cpu.regs.flags & 0x0001) == 0, // JNB/JAE/JNC
                    0x4 => (cpu.regs.flags & 0x0040) != 0, // JE/JZ
                    0x5 => (cpu.regs.flags & 0x0040) == 0, // JNE/JNZ
                    0x6 => (cpu.regs.flags & 0x0041) != 0, // JBE/JNA
                    0x7 => (cpu.regs.flags & 0x0041) == 0, // JNBE/JA
                    0x8 => (cpu.regs.flags & 0x0080) != 0, // JS
                    0x9 => (cpu.regs.flags & 0x0080) == 0, // JNS
                    0xA => (cpu.regs.flags & 0x0004) != 0, // JP/JPE
                    0xB => (cpu.regs.flags & 0x0004) == 0, // JNP/JPO
                    0xC => ((cpu.regs.flags & 0x0080) != 0) != ((cpu.regs.flags & 0x0800) != 0), // JL/JNGE
                    0xD => ((cpu.regs.flags & 0x0080) != 0) == ((cpu.regs.flags & 0x0800) != 0), // JGE/JNL
                    0xE => {
                        ((cpu.regs.flags & 0x0040) != 0)
                            || (((cpu.regs.flags & 0x0080) != 0)
                                != ((cpu.regs.flags & 0x0800) != 0))
                    } // JLE/JNG
                    0xF => {
                        ((cpu.regs.flags & 0x0040) == 0)
                            && (((cpu.regs.flags & 0x0080) != 0)
                                == ((cpu.regs.flags & 0x0800) != 0))
                    } // JG/JNLE
                    _ => false,
                };
                if condition {
                    cpu.regs.ip = cpu.regs.ip.wrapping_add(rel as u16);
                }
            }
            Opcode::JcxzShort(rel) => {
                if cpu.regs.cx == 0 {
                    cpu.regs.ip = cpu.regs.ip.wrapping_add(rel as u16);
                }
            }
            Opcode::Loop(rel) => {
                cpu.regs.cx = cpu.regs.cx.wrapping_sub(1);
                if cpu.regs.cx != 0 {
                    cpu.regs.ip = cpu.regs.ip.wrapping_add(rel as u16);
                }
            }
            Opcode::Loopne(rel) => {
                cpu.regs.cx = cpu.regs.cx.wrapping_sub(1);
                let zf = (cpu.regs.flags & 0x0040) != 0;
                if cpu.regs.cx != 0 && !zf {
                    cpu.regs.ip = cpu.regs.ip.wrapping_add(rel as u16);
                }
            }
            Opcode::Loope(rel) => {
                cpu.regs.cx = cpu.regs.cx.wrapping_sub(1);
                let zf = (cpu.regs.flags & 0x0040) != 0;
                if cpu.regs.cx != 0 && zf {
                    cpu.regs.ip = cpu.regs.ip.wrapping_add(rel as u16);
                }
            }
            Opcode::Cmpsb => {
                let seg = cpu.get_seg(cpu.regs.ds);
                let src = (seg as usize * 16) + cpu.regs.si as usize;
                let dst = (cpu.regs.es as usize * 16) + cpu.regs.di as usize;
                let val1 = cpu.memory.read_u8(src);
                let val2 = cpu.memory.read_u8(dst);
                let res = val1.wrapping_sub(val2);
                cpu.update_flags_isz(res);
                if val1 < val2 {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
                let step = if (cpu.regs.flags & 0x0400) != 0 { 0xFFFF } else { 1 };
                cpu.regs.si = cpu.regs.si.wrapping_add(step);
                cpu.regs.di = cpu.regs.di.wrapping_add(step);
            }
            Opcode::Cmpsw => {
                let seg = cpu.get_seg(cpu.regs.ds);
                let src = (seg as usize * 16) + cpu.regs.si as usize;
                let dst = (cpu.regs.es as usize * 16) + cpu.regs.di as usize;
                let val1 = cpu.memory.read_u16(src);
                let val2 = cpu.memory.read_u16(dst);
                let res = val1.wrapping_sub(val2);
                cpu.update_flags_isz16(res);
                if val1 < val2 {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
                let step = if (cpu.regs.flags & 0x0400) != 0 { 0xFFFE } else { 2 };
                cpu.regs.si = cpu.regs.si.wrapping_add(step);
                cpu.regs.di = cpu.regs.di.wrapping_add(step);
            }
            Opcode::Rep(ref next) => {
                while cpu.regs.cx > 0 {
                    if !next.execute(cpu) {
                        return false;
                    }
                    cpu.regs.cx = cpu.regs.cx.wrapping_sub(1);

                    // Check if we should stop for REPE/REPNE
                    // This is only for CMPS and SCAS
                    let is_cmps_scas = matches!(**next, Opcode::Cmpsb | Opcode::Cmpsw);
                    if is_cmps_scas {
                        let zf = (cpu.regs.flags & 0x0040) != 0;
                        // F3 is REPE (stop if ZF=0)
                        if !zf {
                            break;
                        }
                    }
                }
            }
            Opcode::Movsb => {
                let seg = cpu.get_seg(cpu.regs.ds);
                let src = (seg as usize * 16) + cpu.regs.si as usize;
                let dst = (cpu.regs.es as usize * 16) + cpu.regs.di as usize;
                let val = cpu.memory.read_u8(src);
                cpu.memory.write_u8(dst, val);
                let step = if (cpu.regs.flags & 0x0400) != 0 { 0xFFFF } else { 1 };
                cpu.regs.si = cpu.regs.si.wrapping_add(step);
                cpu.regs.di = cpu.regs.di.wrapping_add(step);
            }
            Opcode::Movsw => {
                let seg = cpu.get_seg(cpu.regs.ds);
                let src = (seg as usize * 16) + cpu.regs.si as usize;
                let dst = (cpu.regs.es as usize * 16) + cpu.regs.di as usize;
                let val = cpu.memory.read_u16(src);
                cpu.memory.write_u16(dst, val);
                let step = if (cpu.regs.flags & 0x0400) != 0 { 0xFFFE } else { 2 };
                cpu.regs.si = cpu.regs.si.wrapping_add(step);
                cpu.regs.di = cpu.regs.di.wrapping_add(step);
            }
            Opcode::Stosb => {
                let dst = (cpu.regs.es as usize * 16) + cpu.regs.di as usize;
                let val = (cpu.regs.ax & 0xFF) as u8;
                cpu.memory.write_u8(dst, val);
                let step = if (cpu.regs.flags & 0x0400) != 0 { 0xFFFF } else { 1 };
                cpu.regs.di = cpu.regs.di.wrapping_add(step);
            }
            Opcode::Stosw => {
                let dst = (cpu.regs.es as usize * 16) + cpu.regs.di as usize;
                let val = cpu.regs.ax;
                cpu.memory.write_u16(dst, val);
                let step = if (cpu.regs.flags & 0x0400) != 0 { 0xFFFE } else { 2 };
                cpu.regs.di = cpu.regs.di.wrapping_add(step);
            }
            Opcode::Lodsb => {
                let seg = cpu.get_seg(cpu.regs.ds);
                let addr = (seg as usize * 16) + cpu.regs.si as usize;
                let val = cpu.memory.read_u8(addr);
                cpu.regs.ax = (cpu.regs.ax & 0xFF00) | (val as u16);
                let step = if (cpu.regs.flags & 0x0400) != 0 { 0xFFFF } else { 1 };
                cpu.regs.si = cpu.regs.si.wrapping_add(step);
            }
            Opcode::InAlImm8(_) => cpu.set_reg8(0, 0xFF),
            Opcode::InAxImm8(_) => cpu.regs.ax = 0xFFFF,
            Opcode::OutImm8Al(_) => {}
            Opcode::OutImm8Ax(_) => {}
            Opcode::InAlDx => cpu.set_reg8(0, 0xFF),
            Opcode::InAxDx => cpu.regs.ax = 0xFFFF,
            Opcode::OutDxAl => {}
            Opcode::OutDxAx => {}
            Opcode::CmpReg16Rm16(modrm) => {
                let val1 = cpu.get_reg16((modrm >> 3) & 0x07);
                let val2 = cpu.get_rm16(modrm);
                let res = val1.wrapping_sub(val2);
                cpu.update_flags_isz16(res);
                if val1 < val2 {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::CmpAccImm8(val2) => {
                let val1 = (cpu.regs.ax & 0xFF) as u8;
                let res = val1.wrapping_sub(val2);
                cpu.update_flags_isz(res);
                if val1 < val2 {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::CmpAccImm16(val2) => {
                let val1 = cpu.regs.ax;
                let res = val1.wrapping_sub(val2);
                cpu.update_flags_isz16(res);
                if val1 < val2 {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::AddAccImm8(imm) => {
                let val = (cpu.regs.ax & 0xFF) as u8;
                let res = val.wrapping_add(imm);
                cpu.regs.ax = (cpu.regs.ax & 0xFF00) | (res as u16);
                cpu.update_flags_isz(res);
                if (val as u16 + imm as u16) > 0xFF {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::AddAccImm16(imm) => {
                let val = cpu.regs.ax;
                let res = val.wrapping_add(imm);
                cpu.regs.ax = res;
                cpu.update_flags_isz16(res);
                if (val as u32 + imm as u32) > 0xFFFF {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::AddRm8Reg8(modrm) => {
                let val1 = cpu.get_rm8(modrm);
                let val2 = cpu.get_reg8((modrm >> 3) & 0x07);
                let res = val1.wrapping_add(val2);
                cpu.set_rm8(modrm, res);
                cpu.update_flags_isz(res);
                if (val1 as u16 + val2 as u16) > 0xFF {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::AddReg8Rm8(modrm) => {
                let val1 = cpu.get_reg8((modrm >> 3) & 0x07);
                let val2 = cpu.get_rm8(modrm);
                let res = val1.wrapping_add(val2);
                cpu.set_reg8((modrm >> 3) & 0x07, res);
                cpu.update_flags_isz(res);
                if (val1 as u16 + val2 as u16) > 0xFF {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::AdcRm8Reg8(modrm) => {
                let val1 = cpu.get_rm8(modrm);
                let val2 = cpu.get_reg8((modrm >> 3) & 0x07);
                let cf = if (cpu.regs.flags & 0x0001) != 0 { 1 } else { 0 };
                let res = val1.wrapping_add(val2).wrapping_add(cf);
                cpu.set_rm8(modrm, res);
                cpu.update_flags_isz(res);
                if (val1 as u16 + val2 as u16 + cf as u16) > 0xFF {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::AdcReg8Rm8(modrm) => {
                let val1 = cpu.get_reg8((modrm >> 3) & 0x07);
                let val2 = cpu.get_rm8(modrm);
                let cf = if (cpu.regs.flags & 0x0001) != 0 { 1 } else { 0 };
                let res = val1.wrapping_add(val2).wrapping_add(cf);
                cpu.set_reg8((modrm >> 3) & 0x07, res);
                cpu.update_flags_isz(res);
                if (val1 as u16 + val2 as u16 + cf as u16) > 0xFF {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::SbbRm8Reg8(modrm) => {
                let val1 = cpu.get_rm8(modrm);
                let val2 = cpu.get_reg8((modrm >> 3) & 0x07);
                let cf = if (cpu.regs.flags & 0x0001) != 0 { 1 } else { 0 };
                let res = val1.wrapping_sub(val2).wrapping_sub(cf);
                cpu.set_rm8(modrm, res);
                cpu.update_flags_isz(res);
                if (val1 as u16) < (val2 as u16 + cf as u16) {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::SbbReg8Rm8(modrm) => {
                let val1 = cpu.get_reg8((modrm >> 3) & 0x07);
                let val2 = cpu.get_rm8(modrm);
                let cf = if (cpu.regs.flags & 0x0001) != 0 { 1 } else { 0 };
                let res = val1.wrapping_sub(val2).wrapping_sub(cf);
                cpu.set_reg8((modrm >> 3) & 0x07, res);
                cpu.update_flags_isz(res);
                if (val1 as u16) < (val2 as u16 + cf as u16) {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::XorRm8Reg8(modrm) => {
                let val1 = cpu.get_rm8(modrm);
                let val2 = cpu.get_reg8((modrm >> 3) & 0x07);
                let res = val1 ^ val2;
                cpu.set_rm8(modrm, res);
                cpu.update_flags_isz(res);
            }
            Opcode::CmpRm8Reg8(modrm) => {
                let val1 = cpu.get_rm8(modrm);
                let val2 = cpu.get_reg8((modrm >> 3) & 0x07);
                let res = val1.wrapping_sub(val2);
                cpu.update_flags_isz(res);
                if val1 < val2 {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::CmpRm16Reg16(modrm) => {
                let val1 = cpu.get_rm16(modrm);
                let val2 = cpu.get_reg16((modrm >> 3) & 0x07);
                let res = val1.wrapping_sub(val2);
                cpu.update_flags_isz16(res);
                if val1 < val2 {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::CmpReg8Rm8(modrm) => {
                let val1 = cpu.get_reg8((modrm >> 3) & 0x07);
                let val2 = cpu.get_rm8(modrm);
                let res = val1.wrapping_sub(val2);
                cpu.update_flags_isz(res);
                if val1 < val2 {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::AdcAccImm8(imm) => {
                let val = (cpu.regs.ax & 0xFF) as u8;
                let cf = if (cpu.regs.flags & 0x0001) != 0 { 1 } else { 0 };
                let res = val.wrapping_add(imm).wrapping_add(cf);
                cpu.regs.ax = (cpu.regs.ax & 0xFF00) | (res as u16);
                cpu.update_flags_isz(res);
                if (val as u16 + imm as u16 + cf as u16) > 0xFF {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::AdcAccImm16(imm) => {
                let val = cpu.regs.ax;
                let cf = if (cpu.regs.flags & 0x0001) != 0 { 1 } else { 0 };
                let res = val.wrapping_add(imm).wrapping_add(cf);
                cpu.regs.ax = res;
                cpu.update_flags_isz16(res);
                if (val as u32 + imm as u32 + cf as u32) > 0xFFFF {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::OrAccImm8(imm) => {
                let val = (cpu.regs.ax & 0xFF) as u8;
                let res = val | imm;
                cpu.regs.ax = (cpu.regs.ax & 0xFF00) | (res as u16);
                cpu.update_flags_isz(res);
                cpu.regs.flags &= !0x0001; // OR clears CF
            }
            Opcode::OrAccImm16(imm) => {
                let val = cpu.regs.ax;
                let res = val | imm;
                cpu.regs.ax = res;
                cpu.update_flags_isz16(res);
                cpu.regs.flags &= !0x0001; // OR clears CF
            }
            Opcode::AndAccImm16(imm) => {
                let val = cpu.regs.ax;
                let res = val & imm;
                cpu.regs.ax = res;
                cpu.update_flags_isz16(res);
                cpu.regs.flags &= !0x0001; // AND clears CF
            }
            Opcode::SubAccImm8(imm) => {
                let val = (cpu.regs.ax & 0xFF) as u8;
                let res = val.wrapping_sub(imm);
                cpu.regs.ax = (cpu.regs.ax & 0xFF00) | (res as u16);
                cpu.update_flags_isz(res);
                if val < imm {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::SubAccImm16(imm) => {
                let val = cpu.regs.ax;
                let res = val.wrapping_sub(imm);
                cpu.regs.ax = res;
                cpu.update_flags_isz16(res);
                if val < imm {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::SbbAccImm8(imm) => {
                let val = (cpu.regs.ax & 0xFF) as u8;
                let cf = if (cpu.regs.flags & 0x0001) != 0 { 1 } else { 0 };
                let res = val.wrapping_sub(imm).wrapping_sub(cf);
                cpu.regs.ax = (cpu.regs.ax & 0xFF00) | (res as u16);
                cpu.update_flags_isz(res);
                if (val as u16) < (imm as u16 + cf as u16) {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::SbbAccImm16(imm) => {
                let val = cpu.regs.ax;
                let cf = if (cpu.regs.flags & 0x0001) != 0 { 1 } else { 0 };
                let res = val.wrapping_sub(imm).wrapping_sub(cf);
                cpu.regs.ax = res;
                cpu.update_flags_isz16(res);
                if (val as u32) < (imm as u32 + cf as u32) {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::XorAccImm8(imm) => {
                let val = (cpu.regs.ax & 0xFF) as u8;
                let res = val ^ imm;
                cpu.regs.ax = (cpu.regs.ax & 0xFF00) | (res as u16);
                cpu.update_flags_isz(res);
                cpu.regs.flags &= !0x0001; // XOR clears CF
            }
            Opcode::XorAccImm16(imm) => {
                let val = cpu.regs.ax;
                let res = val ^ imm;
                cpu.regs.ax = res;
                cpu.update_flags_isz16(res);
                cpu.regs.flags &= !0x0001; // XOR clears CF
            }
            Opcode::AddRm16Reg16(modrm) => {
                let val1 = cpu.get_rm16(modrm);
                let val2 = cpu.get_reg16((modrm >> 3) & 0x07);
                let res = val1.wrapping_add(val2);
                cpu.set_rm16(modrm, res);
                cpu.update_flags_isz16(res);
            }
            Opcode::AddReg16Rm16(modrm) => {
                let val1 = cpu.get_reg16((modrm >> 3) & 0x07);
                let val2 = cpu.get_rm16(modrm);
                let res = val1.wrapping_add(val2);
                cpu.set_reg16((modrm >> 3) & 0x07, res);
                cpu.update_flags_isz16(res);
            }
            Opcode::AdcRm16Reg16(modrm) => {
                let val1 = cpu.get_rm16(modrm);
                let val2 = cpu.get_reg16((modrm >> 3) & 0x07);
                let cf = if (cpu.regs.flags & 0x0001) != 0 { 1u16 } else { 0u16 };
                let res = val1.wrapping_add(val2).wrapping_add(cf);
                cpu.set_rm16(modrm, res);
                cpu.update_flags_isz16(res);
                if (val1 as u32 + val2 as u32 + cf as u32) > 0xFFFF {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::AdcReg16Rm16(modrm) => {
                let val1 = cpu.get_reg16((modrm >> 3) & 0x07);
                let val2 = cpu.get_rm16(modrm);
                let cf = if (cpu.regs.flags & 0x0001) != 0 { 1u16 } else { 0u16 };
                let res = val1.wrapping_add(val2).wrapping_add(cf);
                cpu.set_reg16((modrm >> 3) & 0x07, res);
                cpu.update_flags_isz16(res);
                if (val1 as u32 + val2 as u32 + cf as u32) > 0xFFFF {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::OrRm16Reg16(modrm) => {
                let val1 = cpu.get_rm16(modrm);
                let val2 = cpu.get_reg16((modrm >> 3) & 0x07);
                let res = val1 | val2;
                cpu.set_rm16(modrm, res);
                cpu.update_flags_isz16(res);
            }
            Opcode::OrReg16Rm16(modrm) => {
                let val1 = cpu.get_reg16((modrm >> 3) & 0x07);
                let val2 = cpu.get_rm16(modrm);
                let res = val1 | val2;
                cpu.set_reg16((modrm >> 3) & 0x07, res);
                cpu.update_flags_isz16(res);
            }
            Opcode::OrRm8Reg8(modrm) => {
                let val1 = cpu.get_rm8(modrm);
                let val2 = cpu.get_reg8((modrm >> 3) & 0x07);
                let res = val1 | val2;
                cpu.set_rm8(modrm, res);
                cpu.update_flags_isz(res);
            }
            Opcode::OrReg8Rm8(modrm) => {
                let val1 = cpu.get_reg8((modrm >> 3) & 0x07);
                let val2 = cpu.get_rm8(modrm);
                let res = val1 | val2;
                cpu.set_reg8((modrm >> 3) & 0x07, res);
                cpu.update_flags_isz(res);
            }
            Opcode::AndRm16Reg16(modrm) => {
                let val1 = cpu.get_rm16(modrm);
                let val2 = cpu.get_reg16((modrm >> 3) & 0x07);
                let res = val1 & val2;
                cpu.set_rm16(modrm, res);
                cpu.update_flags_isz16(res);
            }
            Opcode::AndReg16Rm16(modrm) => {
                let val1 = cpu.get_reg16((modrm >> 3) & 0x07);
                let val2 = cpu.get_rm16(modrm);
                let res = val1 & val2;
                cpu.set_reg16((modrm >> 3) & 0x07, res);
                cpu.update_flags_isz16(res);
            }
            Opcode::AndRm8Reg8(modrm) => {
                let val1 = cpu.get_rm8(modrm);
                let val2 = cpu.get_reg8((modrm >> 3) & 0x07);
                let res = val1 & val2;
                cpu.set_rm8(modrm, res);
                cpu.update_flags_isz(res);
            }
            Opcode::AndReg8Rm8(modrm) => {
                let val1 = cpu.get_reg8((modrm >> 3) & 0x07);
                let val2 = cpu.get_rm8(modrm);
                let res = val1 & val2;
                cpu.set_reg8((modrm >> 3) & 0x07, res);
                cpu.update_flags_isz(res);
            }
            Opcode::AndAccImm8(val) => {
                let al = cpu.get_reg8(0);
                let res = al & val;
                cpu.set_reg8(0, res);
                cpu.update_flags_isz(res);
            }
            Opcode::SubRm16Reg16(modrm) => {
                let val1 = cpu.get_rm16(modrm);
                let val2 = cpu.get_reg16((modrm >> 3) & 0x07);
                let res = val1.wrapping_sub(val2);
                cpu.set_rm16(modrm, res);
                cpu.update_flags_isz16(res);
            }
            Opcode::SubReg16Rm16(modrm) => {
                let val1 = cpu.get_reg16((modrm >> 3) & 0x07);
                let val2 = cpu.get_rm16(modrm);
                let res = val1.wrapping_sub(val2);
                cpu.set_reg16((modrm >> 3) & 0x07, res);
                cpu.update_flags_isz16(res);
            }
            Opcode::SubRm8Reg8(modrm) => {
                let val1 = cpu.get_rm8(modrm);
                let val2 = cpu.get_reg8((modrm >> 3) & 0x07);
                let res = val1.wrapping_sub(val2);
                cpu.set_rm8(modrm, res);
                cpu.update_flags_isz(res);
            }
            Opcode::SubReg8Rm8(modrm) => {
                let val1 = cpu.get_reg8((modrm >> 3) & 0x07);
                let val2 = cpu.get_rm8(modrm);
                let res = val1.wrapping_sub(val2);
                cpu.set_reg8((modrm >> 3) & 0x07, res);
                cpu.update_flags_isz(res);
            }
            Opcode::SbbRm16Reg16(modrm) => {
                let val1 = cpu.get_rm16(modrm);
                let val2 = cpu.get_reg16((modrm >> 3) & 0x07);
                let cf = if (cpu.regs.flags & 0x0001) != 0 { 1u16 } else { 0u16 };
                let res = val1.wrapping_sub(val2).wrapping_sub(cf);
                cpu.set_rm16(modrm, res);
                cpu.update_flags_isz16(res);
                if (val1 as u32) < (val2 as u32 + cf as u32) {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::SbbReg16Rm16(modrm) => {
                let val1 = cpu.get_reg16((modrm >> 3) & 0x07);
                let val2 = cpu.get_rm16(modrm);
                let cf = if (cpu.regs.flags & 0x0001) != 0 { 1u16 } else { 0u16 };
                let res = val1.wrapping_sub(val2).wrapping_sub(cf);
                cpu.set_reg16((modrm >> 3) & 0x07, res);
                cpu.update_flags_isz16(res);
                if (val1 as u32) < (val2 as u32 + cf as u32) {
                    cpu.regs.flags |= 0x0001;
                } else {
                    cpu.regs.flags &= !0x0001;
                }
            }
            Opcode::Grp1(opcode, modrm) => {
                let sub_op = (modrm >> 3) & 0x07;
                if opcode == 0x80 {
                    let val1 = cpu.get_rm8(modrm);
                    let val2 = cpu.fetch_u8();
                    match sub_op {
                        0 => {
                            // ADD
                            let res = val1.wrapping_add(val2);
                            cpu.set_rm8(modrm, res);
                            cpu.update_flags_isz(res);
                        }
                        5 => {
                            // SUB
                            let res = val1.wrapping_sub(val2);
                            cpu.set_rm8(modrm, res);
                            cpu.update_flags_isz(res);
                        }
                        7 => {
                            // CMP
                            let res = val1.wrapping_sub(val2);
                            cpu.update_flags_isz(res);
                            if val1 < val2 {
                                cpu.regs.flags |= 0x0001;
                            } else {
                                cpu.regs.flags &= !0x0001;
                            }
                        }
                        _ => {}
                    }
                } else {
                    let val1 = cpu.get_rm16(modrm);
                    let val2 = if opcode == 0x83 {
                        cpu.fetch_u8() as i8 as i16 as u16
                    } else {
                        cpu.fetch_u16()
                    };
                    match sub_op {
                        0 => {
                            // ADD
                            let res = val1.wrapping_add(val2);
                            cpu.set_rm16(modrm, res);
                            cpu.update_flags_isz16(res);
                        }
                        1 => {
                            // OR
                            let res = val1 | val2;
                            cpu.set_rm16(modrm, res);
                            cpu.update_flags_isz16(res);
                        }
                        4 => {
                            // AND
                            let res = val1 & val2;
                            cpu.set_rm16(modrm, res);
                            cpu.update_flags_isz16(res);
                        }
                        5 => {
                            // SUB
                            let res = val1.wrapping_sub(val2);
                            cpu.set_rm16(modrm, res);
                            cpu.update_flags_isz16(res);
                        }
                        6 => {
                            // XOR
                            let res = val1 ^ val2;
                            cpu.set_rm16(modrm, res);
                            cpu.update_flags_isz16(res);
                        }
                        7 => {
                            // CMP
                            let res = val1.wrapping_sub(val2);
                            cpu.update_flags_isz16(res);
                            if val1 < val2 {
                                cpu.regs.flags |= 0x0001;
                            } else {
                                cpu.regs.flags &= !0x0001;
                            }
                        }
                        _ => {}
                    }
                }
            }
            Opcode::Grp2(opcode, modrm) => {
                let count = match opcode {
                    0xD0 | 0xD1 => 1u8,
                    0xD2 | 0xD3 => (cpu.regs.cx & 0xFF) as u8,
                    _ => 0,
                };
                Opcode::execute_grp2(cpu, opcode, modrm, count);
            }
            Opcode::Grp2Imm8(opcode, modrm, count) => {
                Opcode::execute_grp2(cpu, opcode, modrm, count);
            }
            Opcode::Grp3(opcode, modrm) => {
                let sub_op = (modrm >> 3) & 0x07;
                if opcode == 0x16 || opcode == 0xF6 {
                    let val = cpu.get_rm8(modrm);
                    match sub_op {
                        0 => {
                            // TEST imm8
                            let imm = cpu.fetch_u8();
                            cpu.update_flags_isz(val & imm);
                        }
                        2 => cpu.set_rm8(modrm, !val), // NOT
                        3 => {
                            // NEG
                            let res = 0u8.wrapping_sub(val);
                            cpu.set_rm8(modrm, res);
                            cpu.update_flags_isz(res);
                            if val != 0 {
                                cpu.regs.flags |= 0x0001;
                            } else {
                                cpu.regs.flags &= !0x0001;
                            }
                        }
                        _ => {}
                    }
                } else {
                    let val = cpu.get_rm16(modrm);
                    match sub_op {
                        0 => {
                            // TEST imm16
                            let imm = cpu.fetch_u16();
                            cpu.update_flags_isz16(val & imm);
                        }
                        2 => cpu.set_rm16(modrm, !val), // NOT
                        3 => {
                            // NEG
                            let res = 0u16.wrapping_sub(val);
                            cpu.set_rm16(modrm, res);
                            cpu.update_flags_isz16(res);
                            if val != 0 {
                                cpu.regs.flags |= 0x0001;
                            } else {
                                cpu.regs.flags &= !0x0001;
                            }
                        }
                        6 => {
                            // DIV
                            let num = ((cpu.regs.dx as u32) << 16) | (cpu.regs.ax as u32);
                            if val != 0 {
                                let quot = num / (val as u32);
                                let rem = num % (val as u32);
                                cpu.regs.ax = quot as u16;
                                cpu.regs.dx = rem as u16;
                            }
                        }
                        _ => {}
                    }
                }
            }
            Opcode::Pushf => {
                let val = cpu.regs.flags;
                cpu.push16(val);
            }
            Opcode::Popf => cpu.regs.flags = cpu.pop16(),
            Opcode::Sahf => {
                let ah = ((cpu.regs.ax >> 8) & 0xFF) as u16;
                cpu.regs.flags = (cpu.regs.flags & 0xFF00) | ah;
            }
            Opcode::Lahf => {
                let flags = (cpu.regs.flags & 0xFF) as u8;
                cpu.regs.ax = (cpu.regs.ax & 0x00FF) | ((flags as u16) << 8);
            }
            Opcode::TestRm8Reg8(modrm) => {
                let val1 = cpu.get_rm8(modrm);
                let val2 = cpu.get_reg8((modrm >> 3) & 0x07);
                cpu.update_flags_isz(val1 & val2);
            }
            Opcode::TestRm16Reg16(modrm) => {
                let val1 = cpu.get_rm16(modrm);
                let val2 = cpu.get_reg16((modrm >> 3) & 0x07);
                cpu.update_flags_isz16(val1 & val2);
            }
            Opcode::Unknown(op) => {
                glenda::println!(
                    "Unknown Opcode: {:02x} at {:04x}:{:04x}",
                    op,
                    cpu.regs.cs,
                    cpu.regs.ip.wrapping_sub(1)
                );
                return false;
            }
        }
        true
    }

    pub fn execute_grp2(cpu: &mut Cpu, opcode: u8, modrm: u8, count: u8) {
        let sub_op = (modrm >> 3) & 0x7;
        let is_8bit = (opcode & 1) == 0;

        if count == 0 {
            return;
        }

        if is_8bit {
            let mut val = cpu.get_rm8(modrm);
            match sub_op {
                0 => {
                    // ROL
                    for _ in 0..count {
                        let bit = (val & 0x80) != 0;
                        val = (val << 1) | (if bit { 1 } else { 0 });
                        if bit {
                            cpu.regs.flags |= 0x0001;
                        } else {
                            cpu.regs.flags &= !0x0001;
                        }
                    }
                }
                1 => {
                    // ROR
                    for _ in 0..count {
                        let bit = (val & 0x01) != 0;
                        val = (val >> 1) | (if bit { 0x80 } else { 0 });
                        if bit {
                            cpu.regs.flags |= 0x0001;
                        } else {
                            cpu.regs.flags &= !0x0001;
                        }
                    }
                }
                2 => {
                    // RCL
                    for _ in 0..count {
                        let cf = (cpu.regs.flags & 0x0001) != 0;
                        let bit = (val & 0x80) != 0;
                        val = (val << 1) | (if cf { 1 } else { 0 });
                        if bit {
                            cpu.regs.flags |= 0x0001;
                        } else {
                            cpu.regs.flags &= !0x0001;
                        }
                    }
                }
                3 => {
                    // RCR
                    for _ in 0..count {
                        let cf = (cpu.regs.flags & 0x0001) != 0;
                        let bit = (val & 0x01) != 0;
                        val = (val >> 1) | (if cf { 0x80 } else { 0 });
                        if bit {
                            cpu.regs.flags |= 0x0001;
                        } else {
                            cpu.regs.flags &= !0x0001;
                        }
                    }
                }
                4 | 6 => {
                    // SHL/SAL
                    for _ in 0..count {
                        let cf = (val & 0x80) != 0;
                        val <<= 1;
                        if cf {
                            cpu.regs.flags |= 0x0001;
                        } else {
                            cpu.regs.flags &= !0x0001;
                        }
                    }
                    cpu.update_flags_isz(val);
                }
                5 => {
                    // SHR
                    for _ in 0..count {
                        let cf = (val & 0x01) != 0;
                        val >>= 1;
                        if cf {
                            cpu.regs.flags |= 0x0001;
                        } else {
                            cpu.regs.flags &= !0x0001;
                        }
                    }
                    cpu.update_flags_isz(val);
                }
                7 => {
                    // SAR
                    for _ in 0..count {
                        let cf = (val & 0x01) != 0;
                        let msb = val & 0x80;
                        val = (val >> 1) | msb;
                        if cf {
                            cpu.regs.flags |= 0x0001;
                        } else {
                            cpu.regs.flags &= !0x0001;
                        }
                    }
                    cpu.update_flags_isz(val);
                }
                _ => {
                    glenda::println!("Unsupported Grp2(8bit) sub_op: {}", sub_op);
                }
            }
            cpu.set_rm8(modrm, val);
        } else {
            let mut val = cpu.get_rm16(modrm);
            match sub_op {
                0 => {
                    // ROL
                    for _ in 0..count {
                        let bit = (val & 0x8000) != 0;
                        val = (val << 1) | (if bit { 1 } else { 0 });
                        if bit {
                            cpu.regs.flags |= 0x0001;
                        } else {
                            cpu.regs.flags &= !0x0001;
                        }
                    }
                }
                1 => {
                    // ROR
                    for _ in 0..count {
                        let bit = (val & 0x01) != 0;
                        val = (val >> 1) | (if bit { 0x8000 } else { 0 });
                        if bit {
                            cpu.regs.flags |= 0x0001;
                        } else {
                            cpu.regs.flags &= !0x0001;
                        }
                    }
                }
                2 => {
                    // RCL
                    for _ in 0..count {
                        let cf = (cpu.regs.flags & 0x0001) != 0;
                        let bit = (val & 0x8000) != 0;
                        val = (val << 1) | (if cf { 1 } else { 0 });
                        if bit {
                            cpu.regs.flags |= 0x0001;
                        } else {
                            cpu.regs.flags &= !0x0001;
                        }
                    }
                }
                3 => {
                    // RCR
                    for _ in 0..count {
                        let cf = (cpu.regs.flags & 0x0001) != 0;
                        let bit = (val & 0x01) != 0;
                        val = (val >> 1) | (if cf { 0x8000 } else { 0 });
                        if bit {
                            cpu.regs.flags |= 0x0001;
                        } else {
                            cpu.regs.flags &= !0x0001;
                        }
                    }
                }
                4 | 6 => {
                    // SHL/SAL
                    for _ in 0..count {
                        let cf = (val & 0x8000) != 0;
                        val <<= 1;
                        if cf {
                            cpu.regs.flags |= 0x0001;
                        } else {
                            cpu.regs.flags &= !0x0001;
                        }
                    }
                    cpu.update_flags_isz16(val);
                }
                5 => {
                    // SHR
                    for _ in 0..count {
                        let cf = (val & 0x01) != 0;
                        val >>= 1;
                        if cf {
                            cpu.regs.flags |= 0x0001;
                        } else {
                            cpu.regs.flags &= !0x0001;
                        }
                    }
                    cpu.update_flags_isz16(val);
                }
                7 => {
                    // SAR
                    for _ in 0..count {
                        let cf = (val & 0x01) != 0;
                        let msb = val & 0x8000;
                        val = (val >> 1) | msb;
                        if cf {
                            cpu.regs.flags |= 0x0001;
                        } else {
                            cpu.regs.flags &= !0x0001;
                        }
                    }
                    cpu.update_flags_isz16(val);
                }
                _ => {
                    glenda::println!("Unsupported Grp2(16bit) sub_op: {}", sub_op);
                }
            }
            cpu.set_rm16(modrm, val);
        }
    }
}
