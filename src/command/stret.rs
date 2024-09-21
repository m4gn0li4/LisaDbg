use winapi::shared::ntdef::HANDLE;
use crate::ste::get_address;
use crate::{usage, utils::*, ALL_ELM};
use crate::dbg::{memory, BASE_ADDR};
use crate::usage::USAGE_B_RET_VA;

pub fn st_return(linev: &[&str]) {
    if linev.len() < 2 {
        eprintln!("{}", usage::USAGE_B_RET);
        return;
    }
    let addr = match get_address(linev) {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("{ERR_COLOR}{e}{RESET_COLOR}");
            return;
        }
    };
    unsafe {
        if ALL_ELM.break_contain(addr) {
            eprintln!("{ERR_COLOR}you have already placed a breakpoint here {:#x}{RESET_COLOR}", addr);
            return;
        }
    }
    unsafe { ALL_ELM.break_ret.push(addr); }
    println!("a breakpoint will be placed at the return address of the function containing the instruction at address {:#x} + base addr", addr);
}



pub fn handle_stret(linev: &[&str], h_proc: HANDLE) {
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
            if ALL_ELM.break_contain(addr) {
                eprintln!("{ERR_COLOR}you have already placed a breakpoint here {:#x}{RESET_COLOR}", addr);
                return;
            }
            ALL_ELM.break_ret.push(addr);
            memory::breakpoint::set_breakpoint(h_proc, addr + BASE_ADDR);
        }
    } else {
        println!("{}", usage::USAGE_B_RET);
    }
}



pub fn handle_b_ret_va(linev: &[&str]) {
    if linev.len() < 2 {
        eprintln!("{VALID_COLOR}{USAGE_B_RET_VA}{RESET_COLOR}");
        return;
    }
    match str_to::<u64>(linev[1]) {
        Ok(addr) => unsafe {
            if ALL_ELM.break_contain(addr) {
                eprintln!("{ERR_COLOR}you have already placed a breakpoint here {:#x}{RESET_COLOR}", addr);
                return;
            }
            ALL_ELM.break_ret_va.push(addr);
            println!("{VALID_COLOR}a breakpoint will be placed at the return address of the function containing the instruction at address {:#x}{RESET_COLOR}", addr);
        },
        Err(e) => eprintln!("{ERR_COLOR}{e}{RESET_COLOR}"),
    }
}


