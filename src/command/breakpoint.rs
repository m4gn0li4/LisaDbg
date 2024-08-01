use crate::{usage, log::*};
use crate::OPTION;
use winapi::shared::ntdef::HANDLE;
use crate::dbg::{BASE_ADDR, memory};
use crate::symbol::SYMBOLS_V;

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





pub fn get_addr_br(addr_str: &str) -> Result<u64, String> {
    match str_to::<u64>(addr_str) {
        Ok(value) => Ok(value),
        Err(_) => unsafe {
            if let Some(sym) = SYMBOLS_V.symbol_file.iter().find(|s|s.name == addr_str) {
                if sym.offset > 0 {
                    Ok(sym.offset as u64)
                }else {
                    return Err(format!("{ERR_COLOR}the specified symbol cannot have a negative offset{RESET_COLOR}"));
                }
            }else {
                return Err(format!("{ERR_COLOR}invalid target : {}{RESET_COLOR}", addr_str));
            }
        }
    }
}



pub fn handle_breakpoint_proc(linev: &[&str], process_handle: HANDLE) {
    if linev.len() != 2 {
        eprintln!("{}", usage::USAGE_BRPT);
    } else {
        let addr_str = linev[1];
        let addr = match get_addr_br(addr_str) {
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

