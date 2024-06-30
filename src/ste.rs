use winapi::um::winnt::RUNTIME_FUNCTION;
use crate::log::*;
use crate::{OPTION, symbol, usage};
use crate::pefile::function::FUNC_INFO;

pub static mut STE_RETURN_ADDR: Vec<u64> = Vec::new();
pub static mut ST_OVER_ADDR: Vec<u64> = Vec::new();



pub fn st_return(linev: &[&str]) {
    if linev.len() < 2 {
        eprintln!("{}", usage::USAGE_STRET);
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
                STE_RETURN_ADDR.push(addr_func);
                OPTION.breakpoint_addr.push(addr_func);
            }
            println!("{VALID_COLOR}a breakpoint will be placed at each return of the function {RESET_COLOR}");
        },
        None => eprintln!("{ERR_COLOR}unknow target : '{:?}'{RESET_COLOR}", addr_func),
    }
}


pub fn skip(linev: &[&str]) {
    if linev.len() < 2 {
        eprintln!("{}", usage::USAGE_SKIP);
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
                ST_OVER_ADDR.push(addr_func);
            }
            println!("{VALID_COLOR}the function {:#x} will now not be executed{RESET_COLOR}", addr_func);
        },
        None => eprintln!("{ERR_COLOR}unknow target : '{:#x}'{RESET_COLOR}", addr_func),
    }
}



pub fn dskip(linev: &[&str]) {
    process_st_operation(linev, "dskip", unsafe {std::ptr::addr_of_mut!(ST_OVER_ADDR)}, |target| {
        symbol::target_addr_with_name_sym(target)
    });
}

pub fn dret(linev: &[&str]) {
    process_st_operation(linev, "dret", unsafe {std::ptr::addr_of_mut!(STE_RETURN_ADDR)}, |target| {
        symbol::target_addr_with_name_sym(target)
    });
}

pub fn process_st_operation<F>(linev: &[&str], operation_name: &str, addr_ptr: *mut Vec<u64>, target_addr: F) where F: FnOnce(&str) -> u64 {
    if linev.len() != 2 {
        println!("{}", usage::USAGE_SKIP.replace("skip", operation_name));
        return;
    }

    let target = linev[1];
    let addr = match str_to::<u64>(target) {
        Ok(value) => value,
        Err(_) => {
            let addr = target_addr(target);
            if addr == 0 {
                eprintln!("{ERR_COLOR}invalid target : {target}");
                return;
            }
            addr
        }
    };

    unsafe {
        let addr_vec = &mut *addr_ptr;
        addr_vec.retain(|&a| a != addr);
    }
    println!("{VALID_COLOR}{target} has been retained successfully{RESET_COLOR}");
}


fn get_(linev: &[&str]) -> usize {
    if linev.len() > 1 {
        linev.len() - 2
    } else {
        0
    }
}

fn get_address(linev: &[&str]) -> Result<u64, String> {
    let flag = linev[get_(linev)];
    let target = *linev.last().unwrap();
    if flag == "-a" || flag == "--address" {
        str_to::<u64>(target).map_err(|e| e.to_string())
    } else {
        let addr_func = symbol::target_addr_with_name_sym(target);
        if addr_func == 0 {
            Err(format!("unknown symbol: '{}'", target))
        } else {
            Ok(addr_func)
        }
    }
}



fn find_func_by_addr(addr: u64) -> Option<&'static RUNTIME_FUNCTION> {
    unsafe { FUNC_INFO.iter().find(|func| func.BeginAddress == addr as u32) }
}