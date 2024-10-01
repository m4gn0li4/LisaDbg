use crate::dbg::memory::breakpoint::set_breakpoint;
use crate::dbg::{memory, BASE_ADDR};
use crate::{usage, utils::*, ALL_ELM};
use winapi::shared::ntdef::HANDLE;
use winapi::um::winnt::{CONTEXT, WOW64_CONTEXT};

pub fn handle_breakpts(linev: &[&str]) {
    if linev.len() == 2 {
        let addr = match get_addr_br(linev[1]) {
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
            ALL_ELM.break_rva.push(addr);
        }
        println!("{VALID_COLOR}Breakpoints are set at address {:#x}{RESET_COLOR}", addr);
    } else {
        eprintln!("{}", usage::USAGE_BRPT);
    }
}

pub fn handle_break_va(linev: &[&str]) {
    if linev.len() != 2 {
        println!("b-va <va>");
        return;
    }
    match str_to::<u64>(linev[1]) {
        Ok(addr) => unsafe {
            if ALL_ELM.break_contain(addr) {
                eprintln!("{ERR_COLOR}you have already placed a breakpoint here {:#x}{RESET_COLOR}", addr);
                return;
            }
            ALL_ELM.break_va.push(addr);
            println!("{VALID_COLOR}breakpoints are set at address {:#x}{RESET_COLOR}", addr);
        },
        Err(e) => eprintln!("{ERR_COLOR}{e}{RESET_COLOR}"),
    }
}

pub fn handle_b_va_proc(linev: &[&str], h_proc: HANDLE) {
    if linev.len() != 2 {
        println!("b-va <address>");
        return;
    }
    match str_to::<u64>(linev[1]) {
        Ok(addr) => unsafe {
            if ALL_ELM.break_contain(addr) {
                eprintln!("{ERR_COLOR}you have already placed a breakpoint here : {:#x}{RESET_COLOR}", addr);
                return;
            }
            ALL_ELM.break_va.push(addr);
            set_breakpoint(h_proc, addr)
        },
        Err(e) => eprintln!("{ERR_COLOR}{e}{RESET_COLOR}"),
    }
}

pub fn handle_breakpoint_proc(linev: &[&str], h_proc: HANDLE, ctx: CONTEXT) {
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
            if ALL_ELM.break_contain(addr) {
                eprintln!("{ERR_COLOR}you have already placed a breakpoint here : {:#x}{RESET_COLOR}", addr);
                return;
            }
            ALL_ELM.break_rva.push(addr);
            set_breakpoint(h_proc, addr + BASE_ADDR)
        }
    }
}

pub fn handle_breakpoint_proc32(linev: &[&str], h_proc: HANDLE, ctx: WOW64_CONTEXT) {
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
            if ALL_ELM.break_contain(addr as u64) {
                eprintln!("{ERR_COLOR}you have already placed a breakpoint here{RESET_COLOR}");
                return;
            }
            ALL_ELM.break_rva.push(addr as u64);
            set_breakpoint(h_proc, addr as u64 + BASE_ADDR)
        }
    }
}

pub fn handle_restore_breakpoint_proc(linev: &[&str], h_proc: HANDLE) {
    if linev.len() == 2 {
        let addr_str = linev[1];
        let addr = match get_addr_br(addr_str) {
            Ok(value) => value,
            Err(e) => {
                eprintln!("{e}");
                return;
            }
        };
        unsafe { memory::breakpoint::restore_byte_of_brkpt(h_proc, addr + BASE_ADDR) }
    }
}
