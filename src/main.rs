#![no_std]
#![no_main]

extern crate alloc;
use glenda::println;
use glenda::runtime::bootinfo::BootInfo;
use glenda::runtime::initrd::Initrd;
use glenda::runtime::{BOOTINFO_VA, bootinfo};

mod cpu;
use cpu::cpu::Cpu;

fn read_line(buf: &mut [u8]) -> usize {
    let mut i = 0;
    while i < buf.len() - 1 {
        let c = glenda::runtime::KERNEL_CAP.console_get_char();
        if c == '\r' || c == '\n' {
            glenda::print!("\n");
            break;
        }
        if c == '\x08' || c == '\x7F' {
            // Backspace
            if i > 0 {
                i -= 1;
                glenda::print!("\x08 \x08");
            }
            continue;
        }
        if c as u8 >= 32 && c as u8 <= 126 {
            buf[i] = c as u8;
            i += 1;
            glenda::print!("{}", c);
        }
    }
    i
}

#[unsafe(no_mangle)]
fn main() -> usize {
    println!("Starting DOSBox Simple Emulator Shell...");

    // 1. Get BootInfo to locate Initrd
    let bootinfo = unsafe { &*(BOOTINFO_VA as *const BootInfo) };
    if bootinfo.magic != bootinfo::BOOTINFO_MAGIC {
        println!("Error: Invalid BootInfo magic!");
        return 1;
    }

    // 2. Parse Initrd
    let initrd_start = bootinfo.initrd_start;
    let initrd_size = bootinfo.initrd_size;
    if initrd_start == 0 || initrd_size == 0 {
        println!("Error: Initrd not found!");
        return 1;
    }

    let initrd_slice =
        unsafe { core::slice::from_raw_parts(initrd_start as *const u8, initrd_size) };

    let initrd = match Initrd::new(initrd_slice) {
        Ok(i) => i,
        Err(e) => {
            println!("Error parsing Initrd: {}", e);
            return 1;
        }
    };

    println!("DOSBox Shell Initialized. Type 'help' for commands.");

    let mut cpu: Option<Cpu> = None;
    let mut current_floppy_name: Option<alloc::string::String> = None;
    let mut input_buf = [0u8; 64];

    loop {
        glenda::print!("dosbox> ");
        let len = read_line(&mut input_buf);
        if len == 0 {
            continue;
        }

        let cmd_str = match core::str::from_utf8(&input_buf[..len]) {
            Ok(s) => s.trim(),
            Err(_) => continue,
        };

        let mut parts = cmd_str.split_whitespace();
        let cmd = parts.next().unwrap_or("");
        let args: alloc::vec::Vec<&str> = parts.collect();

        match cmd {
            "help" => {
                println!("Available commands:");
                println!("  ls             - List files in Initrd");
                println!("  load <file>    - Load a floppy image");
                println!("  run            - Start/Resume simulation");
                println!("  reset          - Reset CPU state");
                println!("  dump           - Dump registers");
                println!("  info           - Show current floppy info");
                println!("  exit           - Exit DOSBox");
            }
            "ls" => {
                println!("Initrd Files:");
                for entry in &initrd.entries {
                    println!("  {} ({} bytes, type {:?})", entry.name, entry.size, entry.type_);
                }
            }
            "load" => {
                if args.len() < 1 {
                    println!("Usage: load <filename>");
                    continue;
                }
                let filename = args[0];
                let mut found = false;
                for entry in &initrd.entries {
                    if entry.name == filename {
                        let start = entry.offset;
                        let end = start + entry.size;
                        if end <= initrd.data.len() {
                            let data = &initrd.data[start..end];
                            cpu = Some(Cpu::new(data));
                            current_floppy_name = Some(alloc::string::String::from(filename));
                            println!("Loaded {} successfully.", filename);
                            found = true;
                            break;
                        }
                    }
                }
                if !found {
                    println!("File '{}' not found in Initrd.", filename);
                }
            }
            "run" => {
                if let Some(ref mut c) = cpu {
                    println!("Starting simulation...");
                    let mut steps = 0;
                    while c.step() {
                        steps += 1;
                    }
                    println!("\nSimulation Halted (Steps: {})", steps);
                    c.dump_registers();
                } else {
                    println!("No floppy loaded. Use 'load <file>' first.");
                }
            }
            "reset" => {
                if let Some(ref name) = current_floppy_name {
                    // Reload current floppy
                    for entry in &initrd.entries {
                        if entry.name == *name {
                            cpu = Some(Cpu::new(
                                &initrd.data[entry.offset..entry.offset + entry.size],
                            ));
                            println!("CPU Reset and {} reloaded.", name);
                            break;
                        }
                    }
                } else {
                    println!("Nothing to reset.");
                }
            }
            "dump" => {
                if let Some(ref c) = cpu {
                    c.dump_registers();
                } else {
                    println!("No CPU instance.");
                }
            }
            "info" => {
                if let Some(ref name) = current_floppy_name {
                    println!("Current Floppy: {}", name);
                } else {
                    println!("No floppy loaded.");
                }
            }
            "exit" => {
                println!("Exiting DOSBox.");
                break;
            }
            "" => {}
            _ => {
                println!("Unknown command: {}", cmd);
            }
        }
    }

    // Halt
    0
}
