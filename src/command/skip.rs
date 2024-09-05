use crate::usage;
use crate::utils::*;


pub static mut SKIP_ADDR: Vec<u64> = Vec::new();

pub fn skip(linev: &[&str]) {
    if linev.len() < 2 {
        eprintln!("{}", usage::USAGE_SKIP);
        return;
    }
    let addr_func = match crate::ste::get_address(linev) {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("{ERR_COLOR}{e}{RESET_COLOR}");
            return;
        }
    };

    match crate::ste::find_func_by_addr(addr_func) {
        Some(_) => {
            unsafe {
                SKIP_ADDR.push(addr_func);
            }
            println!("{VALID_COLOR}the function {:#x} will now not be executed{RESET_COLOR}", addr_func);
        },
        None => eprintln!("{ERR_COLOR}unknow target : '{:#x}'{RESET_COLOR}", addr_func),
    }
}


