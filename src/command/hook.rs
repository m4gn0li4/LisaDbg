use winapi::um::winnt::HANDLE;
use crate::utils::*;
use crate::{usage, ALL_ELM};
use crate::dbg::{memory, BASE_ADDR};
use crate::symbol::SYMBOLS_V;

#[derive(Debug, Default, Copy, Clone)]
pub struct Hook {
    pub target: u64,
    pub replacen: u64,
}

fn get_addr_or_symbol(linev: &[&str], idx: usize) -> Option<u64> {
    match str_to::<u64>(linev[idx]) {
        Ok(addr) => Some(addr),
        Err(_) => {
            if let Some(sym) = unsafe { SYMBOLS_V.symbol_file.iter().find(|s| s.name == linev[idx]) } {
                if sym.offset > 0 {
                    Some(sym.offset as u64)
                } else {
                    eprintln!("{ERR_COLOR}You cannot specify local symbols{RESET_COLOR}");
                    None
                }
            } else {
                eprintln!("{ERR_COLOR}Invalid target: {}{RESET_COLOR}", linev[idx]);
                None
            }
        }
    }
}

fn set_hook(addr1: u64, addr2: u64, h_proc: Option<HANDLE>) {
    if let Some(h_proc) = h_proc {
        unsafe { memory::breakpoint::set_breakpoint(h_proc, addr1 + BASE_ADDR); }
    }
    unsafe { ALL_ELM.hook.push(Hook { target: addr1, replacen: addr2 }); }
    println!("{VALID_COLOR}Now when the program reaches rva {:#x}, it will be redirected to rva {:#x}{RESET_COLOR}", addr1, addr2);
}



pub fn hook(linev: &[&str]) {
    if linev.len() < 3 {
        eprintln!("{}", usage::USAGE_HOOK);
        return;
    }

    let addr1 = match get_addr_or_symbol(linev, 1) {
        Some(addr) => addr,
        None => return,
    };

    let addr2 = match get_addr_or_symbol(linev, 2) {
        Some(addr) => addr,
        None => return,
    };

    set_hook(addr1, addr2, None);
}



pub fn handle_hook_proc(linev: &[&str], h_proc: HANDLE) {
    if linev.len() < 3 {
        eprintln!("{}", usage::USAGE_HOOK);
        return;
    }

    let addr1 = match get_addr_or_symbol(linev, 1) {
        Some(addr) => addr,
        None => return,
    };

    let addr2 = match get_addr_or_symbol(linev, 2) {
        Some(addr) => addr,
        None => return,
    };

    set_hook(addr1, addr2, Some(h_proc));
}
