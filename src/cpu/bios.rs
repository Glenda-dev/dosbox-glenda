use crate::cpu::cpu::Cpu;
use glenda::print;

pub fn handle_interrupt(cpu: &mut Cpu, int_no: u8) -> bool {
    match int_no {
        0x10 => {
            // BIOS Video Service
            let ah = (cpu.regs.ax >> 8) as u8;
            match ah {
                0x0E => {
                    // Teletype output
                    let char_code = (cpu.regs.ax & 0xFF) as u8;
                    match char_code {
                        0x0A => print!("\n"),
                        0x0D => print!("\r"),
                        0x07 => {}              // Bell - ignore
                        0x08 => print!("\x08"), // Backspace
                        _ => {
                            if char_code >= 32 && char_code <= 126 {
                                print!("{}", char_code as char);
                            }
                        }
                    }
                }
                0x03 => {
                    // Get cursor position
                    cpu.regs.ax = 0;
                    cpu.regs.cx = 0x0607;
                    cpu.regs.dx = 0;
                }
                0x0F => {
                    // Get video mode
                    cpu.regs.ax = 0x0003;
                    cpu.regs.bx &= 0x00FF;
                }
                _ => {}
            }
        }
        0x11 => {
            // Get Equipment List
            cpu.regs.ax = 0x0141;
        }
        0x13 => {
            // BIOS Disk Service
            let ah = (cpu.regs.ax >> 8) as u8;
            match ah {
                0x00 => {
                    cpu.regs.flags &= !0x0001;
                    cpu.regs.ax &= 0x00FF;
                }
                0x02 => {
                    // Read sectors
                    let num_sectors = (cpu.regs.ax & 0xFF) as usize;
                    let cylinder = (((cpu.regs.cx & 0xC0) << 2) | (cpu.regs.cx >> 8)) as usize;
                    let sector = (cpu.regs.cx & 0x3F) as usize;
                    let head = (cpu.regs.dx >> 8) as usize;

                    let spt = if cpu.disk_image.len() == 1474560 { 18 } else { 9 };
                    let lba = (cylinder * 2 + head) * spt + (sector - 1);
                    let mut offset = lba * 512;
                    let mut target_addr = (cpu.regs.es as usize * 16) + cpu.regs.bx as usize;

                    for _ in 0..num_sectors {
                        if offset + 512 <= cpu.disk_image.len() {
                            for i in 0..512 {
                                let b = cpu.disk_image[offset + i];
                                cpu.memory.write_u8(target_addr + i, b);
                            }
                            offset += 512;
                            target_addr += 512;
                        }
                    }

                    cpu.regs.flags &= !0x0001;
                    cpu.regs.ax = cpu.regs.ax & 0x00FF;
                }
                0x08 => {
                    let spt = if cpu.disk_image.len() == 1474560 { 18 } else { 9 };
                    let cylinders = 80;
                    cpu.regs.ax = 0;
                    cpu.regs.bx = 0x0004;
                    cpu.regs.cx =
                        (((cylinders - 1) & 0xFF) << 8) | (((cylinders - 1) & 0x300) >> 2) | spt;
                    cpu.regs.dx = 0x0102;
                    cpu.regs.flags &= !0x0001;
                }
                _ => {
                    cpu.regs.flags |= 0x0001;
                }
            }
        }
        0x16 => {
            let ah = (cpu.regs.ax >> 8) as u8;
            match ah {
                0x00 | 0x10 => {
                    cpu.regs.ax = 0x1C0D;
                }
                0x01 | 0x11 => {
                    cpu.regs.flags |= 0x0040;
                }
                _ => {}
            }
        }
        0x19 => {
            cpu.regs.cs = 0x0000;
            cpu.regs.ip = 0x7C00;
            cpu.regs.dx = 0x0000;
            cpu.regs.sp = 0x7C00;
            return false;
        }
        0x21 => {
            let ah = (cpu.regs.ax >> 8) as u8;
            match ah {
                0x09 => {
                    let mut addr = (cpu.regs.ds as usize * 16) + cpu.regs.dx as usize;
                    loop {
                        let b = cpu.memory.read_u8(addr);
                        if b == b'$' {
                            break;
                        }
                        print!("{}", b as char);
                        addr += 1;
                    }
                }
                0x4C => {
                    cpu.regs.ip = 0;
                }
                _ => return false,
            }
        }
        0x20 => {
            cpu.regs.ip = 0;
        }
        _ => return false,
    }
    true
}
