use crate::{usage, utils::*};
use crate::OPTION;
use winapi::shared::ntdef::HANDLE;
use winapi::um::winnt::{CONTEXT, WOW64_CONTEXT};
use crate::dbg::{BASE_ADDR, memory};

pub fn handle_breakpts(linev: &[&str]) {
    if linev.len() == 2 {
        let addr = match get_addr_br(linev[1]) {
            Ok(value) => value,
            Err(e) => {
                eprintln!("{e}");
                return;
            }
        };
        unsafe { OPTION.breakpoint_addr.push(addr); }
        println!("{VALID_COLOR}Breakpoints are set at address {:#x}{RESET_COLOR}", addr);
    }else {
        eprintln!("{}", usage::USAGE_BRPT);
    }
}



pub fn handle_breakpoint_proc(linev: &[&str], process_handle: HANDLE, ctx: CONTEXT) {
    if linev.len() != 2 {
        eprintln!("{}", usage::USAGE_BRPT);
    } else {
        let addr_str = linev[1];
        let addr = match get_addr_va(addr_str, ctx) {
            Ok(value) => value,
            Err(e) => {
                eprintln!("{e}");
                return;
            }
        };
        unsafe {
            OPTION.breakpoint_addr.push(addr);
            memory::breakpoint::set_breakpoint(process_handle, addr)
        }
    }
}


pub fn handle_breakpoint_proc32(linev: &[&str], process_handle: HANDLE, ctx: WOW64_CONTEXT) {
    if linev.len() != 2 {
        eprintln!("{}", usage::USAGE_BRPT);
    } else {
        let addr_str = linev[1];
        let addr = match get_addr_va32(addr_str, ctx) {
            Ok(value) => value,
            Err(e) => {
                eprintln!("{e}");
                return;
            }
        };
        unsafe {
            OPTION.breakpoint_addr.push(addr as u64);
            memory::breakpoint::set_breakpoint(process_handle, addr as u64)
        }
    }
}





pub fn handle_restore_breakpoint_proc(linev: &[&str], process_handle: HANDLE) {
    if linev.len() == 2 {
        let addr_str = linev[1];
        let addr = match get_addr_br(addr_str) {
            Ok(value) => value,
            Err(e) => {
                eprintln!("{e}");
                return;
            }
        };
        unsafe { memory::breakpoint::restore_byte_of_brkpt(process_handle, addr + BASE_ADDR) }
    }
}

