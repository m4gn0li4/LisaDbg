pub mod deref_mem;
mod modifier;
pub mod usages;
mod info_reg;

use std::io::{self, Write};
use std::str;
use iced_x86::{Decoder, DecoderOptions, Instruction};
use winapi::shared::minwindef::LPVOID;
use winapi::um::processthreadsapi;
use winapi::shared::ntdef::HANDLE;
use winapi::um::memoryapi::ReadProcessMemory;
use winapi::um::winnt::CONTEXT;
use crate::{command, ste, symbol, usage};
use crate::dbg::{BASE_ADDR, memory, ST_BEGIN_INFO};
use crate::log::*;
use crate::ste::{ST_OVER_ADDR, STE_RETURN_ADDR};

pub fn cmd_wait(ctx: &mut CONTEXT, process_handle: HANDLE, continue_debugging: &mut bool) {
    let mut input = String::new();
    let mut stop_process = false;

    while !stop_process {
        input.clear();
        print!("\x1b[38;5;129m>> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        print!("{RESET_COLOR}");
        io::stdout().flush().unwrap();
        let linev: Vec<&str> = input.split_whitespace().collect();
        let cmd = linev.first();
        match cmd {
            Some(&"c") | Some(&"continue") | Some(&"run") => break,
            Some(&"v") | Some(&"value") => unsafe { info_reg::handle_reg(&linev, *ctx) },
            Some(&"deref") => deref_mem::handle_deref(&linev, *ctx, process_handle),
            Some(&"setr") | Some(&"setreg") => modifier::register::handle_set_register(&linev, ctx),
            Some(&"q") | Some(&"quit") | Some(&"break") => handle_quit(&mut input, process_handle, continue_debugging, &mut stop_process),
            Some(&"base-addr") | Some(&"ba") => println!("base address : {VALUE_COLOR}{:#x}{RESET_COLOR}", unsafe { BASE_ADDR }),
            Some(&"setm") | Some(&"setmemory") => modifier::set_memory::handle_set_memory(process_handle, *ctx, &linev),
            Some(&"b") | Some(&"breakpoint") => handle_breakpoint(&linev, process_handle),
            Some(&"rb") => handle_restore_breakpoint(&linev, process_handle),
            Some(&"reset") => command::reset::handle_reset(&linev),
            Some(&"cva") => command::with_va::handle_calcule_va(&linev),
            Some(&"b-sp") => handle_b_sp(),
            Some(&"ret-addr") | Some(&"raddr") => handle_ret_addr(),
            Some(&"ret") => handle_ret(ctx),
            Some(&"stret") => handle_stret(&linev, process_handle),
            Some(&"skip") => handle_skip(&linev, process_handle),
            Some(&"dskip") => handle_dskip(&linev, process_handle),
            Some(&"dret") => handle_dret(&linev, process_handle),
            Some(&"view") => command::viewing::view_brpkt(&linev),
            Some(&"help") => usages::help(),
            Some(&"disasm") => handle_disasm(&linev, process_handle),
            None => eprintln!("{ERR_COLOR}Please enter a command{RESET_COLOR}"),
            _ => eprintln!("{ERR_COLOR}Unknown command: {}{RESET_COLOR}", cmd.unwrap()),
        }
    }
}




fn handle_disasm(linev: &[&str], process_handle: HANDLE) {
    if linev.len() != 3 {
        println!("{}", usages::USAGE_DISASM);
        return;
    }

    let addr_str = linev[1];
    let count_str = linev[2];

    let addr = match str_to::<u64>(addr_str) {
        Ok(addr) => addr,
        Err(e) => {
            println!("{ERR_COLOR}Invalid address: {e}{RESET_COLOR}");
            return;
        }
    };

    let count = match str_to::<usize>(count_str) {
        Ok(count) => count,
        Err(e) => {
            println!("{ERR_COLOR}invalid count: {e}{RESET_COLOR}");
            return;
        }
    };
    let mut buffer = vec![0u8; 2093];
    unsafe {
        if ReadProcessMemory(process_handle, addr as LPVOID, buffer.as_mut_ptr() as *mut _, 2093, std::ptr::null_mut()) == 0 {
            println!("Failed to read process memory.");
            return;
        }
    }

    let mut decoder = Decoder::with_ip(64, &buffer, addr, DecoderOptions::NONE);
    let mut instruction = Instruction::default();
    let mut i = 0;
    while decoder.can_decode() && i < count {
        decoder.decode_out(&mut instruction);
        println!("{VALUE_COLOR}{:#x}: {instruction}{RESET_COLOR}", instruction.ip());
        i += 1;
    }
}



fn handle_quit(input: &mut String, process_handle: HANDLE, continue_debugging: &mut bool, stop_process: &mut bool) {
    input.clear();
    print!("Are you sure to stop this process? [y/n] : ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(input).unwrap();
    if input.trim() == "y" {
        unsafe { processthreadsapi::TerminateProcess(process_handle, 0) };
        *continue_debugging = false;
        *stop_process = true;
    }
}

fn handle_breakpoint(linev: &[&str], process_handle: HANDLE) {
    if linev.len() != 2 {
        eprintln!("{}", usage::USAGE_BRPT);
    } else {
        let addr_str = linev[1];
        let addr = match str_to::<u64>(addr_str) {
            Ok(value) => value,
            Err(_) => {
                let addr = symbol::target_addr_with_name_sym(addr_str);
                if addr == 0 {
                    eprintln!("{ERR_COLOR}invalid target : {}{RESET_COLOR}", addr_str);
                    return;
                }
                addr
            }
        };

        unsafe {
            crate::OPTION.breakpoint_addr.push(addr);
            memory::set_breakpoint(process_handle, BASE_ADDR as LPVOID, addr)
        }
    }
}

fn handle_restore_breakpoint(linev: &[&str], process_handle: HANDLE) {
    if linev.len() == 2 {
        let addr_str = linev[1];
        let addr = match str_to::<u64>(addr_str) {
            Ok(value) => value,
            Err(_) => {
                let addr = symbol::target_addr_with_name_sym(addr_str);
                if addr == 0 {
                    eprintln!("{ERR_COLOR}target is invalid : '{addr_str}'");
                    return;
                }
                addr
            }
        };

        unsafe { memory::restore_byte_of_brkpt(process_handle, addr + BASE_ADDR) }
    } else {
        println!("{}", usage::USAGE_BRPT.replace("breakpoint", "retain-breakpoint"));
    }
}

fn handle_b_sp() {
    unsafe {
        if ST_BEGIN_INFO.sp != 0 {
            println!("rsp in begin function : {VALUE_COLOR}{:#x}{RESET_COLOR}", ST_BEGIN_INFO.sp)
        } else {
            print_msg_w();
        }
    }
}

fn handle_ret_addr() {
    unsafe {
        if ST_BEGIN_INFO.ret_addr == 0 {
            print_msg_w();
        } else {
            println!("ret address : {VALUE_COLOR}{:#x}{RESET_COLOR}", ST_BEGIN_INFO.ret_addr);
        }
    }
}

fn handle_ret(ctx: &mut CONTEXT) {
    unsafe {
        if ST_BEGIN_INFO.ret_addr != 0 {
            ctx.Rip = ST_BEGIN_INFO.ret_addr;
            ctx.Rsp -= 8;
            println!("{VALID_COLOR}now rip points to the address : {VALUE_COLOR}{:#x}{RESET_COLOR}\n{VALID_COLOR}and rsp was decremented by 8 : {VALUE_COLOR}{:#x}{RESET_COLOR}", ST_BEGIN_INFO.ret_addr, ctx.Rsp);
        } else {
            print_msg_w();
        }
    }
}

fn handle_stret(linev: &[&str], process_handle: HANDLE) {
    if linev.len() == 2 {
        let target = linev[1];
        let addr = symbol::target_addr_with_name_sym(target);
        if addr == 0 {
            eprintln!("{ERR_COLOR}unknown symbol : {target}{RESET_COLOR}");
        } else {
            unsafe {
                STE_RETURN_ADDR.push(addr);
                memory::set_breakpoint(process_handle, BASE_ADDR as LPVOID, addr);
            }
        }
    } else {
        println!("{}", usage::USAGE_STRET);
    }
}

fn handle_skip(linev: &[&str], process_handle: HANDLE) {
    if linev.len() == 2 {
        let target = linev[1];
        let addr = symbol::target_addr_with_name_sym(target);
        if addr == 0 {
            eprintln!("{ERR_COLOR}unknown symbol : {target}");
        } else {
            unsafe {
                ste::ST_OVER_ADDR.push(addr);
                memory::set_addr_over(process_handle, addr);
            }
        }
    } else {
        println!("{}", usage::USAGE_SKIP);
    }
}

fn handle_command(linev: &[&str], process_handle: HANDLE, usage_str: &str, addr_set: &mut Vec<u64>) {
    if linev.len() != 2 {
        println!("{}", usage_str);
        return;
    }
    let target = linev[1];
    let addr = match str_to::<u64>(target) {
        Ok(value) => value,
        Err(_) => {
            let addr = symbol::target_addr_with_name_sym(target);
            if addr == 0 {
                eprintln!("{ERR_COLOR}invalid target : {target}");
                return;
            }
            addr
        }
    };
    unsafe {
        addr_set.retain(|&s| s != addr);
        memory::restore_byte_of_brkpt(process_handle, addr + BASE_ADDR);
    }
}


fn handle_dskip(linev: &[&str], process_handle: HANDLE) {
    handle_command(linev, process_handle, &usage::USAGE_SKIP.replace("skip", "dskip"), unsafe {&mut *std::ptr::addr_of_mut!(ST_OVER_ADDR)});
}

fn handle_dret(linev: &[&str], process_handle: HANDLE) {
    handle_command(linev, process_handle, &usage::USAGE_STRET.replace("stret", "dret"), unsafe { &mut *std::ptr::addr_of_mut!(STE_RETURN_ADDR)});
}



fn print_msg_w() {
    eprintln!("{WAR_COLOR}the function in which you are located was not specified with the \"sret\" option, therefore information such as return addresses and other information relating to the stack are not available for the latter{RESET_COLOR}");
}
