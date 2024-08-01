use winapi::shared::ntdef::HANDLE;
use crate::ste::{find_func_by_addr, get_address};
use crate::{OPTION, usage, log::*};
use crate::command::breakpoint::get_addr_br;
use crate::dbg::memory;

pub static mut BREAK_RET: Vec<u64> = Vec::new();

pub fn st_return(linev: &[&str]) {
    if linev.len() < 2 {
        eprintln!("{}", usage::USAGE_B_RET);
        return;
    }
    let addr_func = match get_address(linev) {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("{ERR_COLOR}{e}{RESET_COLOR}");
            return;
        }
    };

    match find_func_by_addr(addr_func) {
        Some(_) => {
            unsafe {
                BREAK_RET.push(addr_func);
                OPTION.breakpoint_addr.push(addr_func);
            }
            println!("{VALID_COLOR}a breakpoint will be placed at each return of the function {RESET_COLOR}");
        },
        None => eprintln!("{ERR_COLOR}unknow target : '{:#x}'{RESET_COLOR}", addr_func),
    }
}



pub fn handle_stret(linev: &[&str], process_handle: HANDLE) {
    if linev.len() == 2 {
        let target = linev[1];
        let addr = match get_addr_br(target) {
            Ok(value) => value,
            Err(e) => {
                eprintln!("{e}");
                return;
            }
        };
        unsafe {
            BREAK_RET.push(addr);
            memory::breakpoint::set_breakpoint(process_handle, addr);
        }
    } else {
        println!("{}", usage::USAGE_B_RET);
    }
}


