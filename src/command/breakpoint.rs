use crate::{symbol, usage};
use crate::log::*;
use crate::OPTION;
use anyhow::{Result, Error};

pub fn handle_breakpts(linev: &[&str]) {
    if linev.len() == 2 {
        match str_to(linev[1]) {
            Ok(addr) => {
                unsafe {
                    OPTION.breakpoint_addr.push(addr);
                    println!("{VALID_COLOR}Breakpoints are set at addresses : {:#x}{RESET_COLOR}", addr);
                }
            }
            Err(_) =>  {
                let addr = symbol::target_addr_with_name_sym(linev[1]);
                if addr == 0 {
                    eprintln!("{ERR_COLOR}invalid target : {}", linev[1]);
                    return;
                }
                unsafe {
                    OPTION.breakpoint_addr.push(addr);
                    println!("{VALID_COLOR}Breakpoints are set at addresses : {:#x}{RESET_COLOR}", addr);
                }
            },
        }
    }else {
        eprintln!("{}", usage::USAGE_BRPT);
    }
}



pub fn handle_retain_breakpoint(linev: &[&str]) {
    if linev.len() == 2 {
        let addr_result: Result<u64, Error> = match str_to(linev[1]) {
            Ok(addr) => Ok(addr),
            Err(_) => {
                let addr = symbol::target_addr_with_name_sym(linev[1]);
                if addr == 0 {
                    eprintln!("{ERR_COLOR}invalid target : {}{RESET_COLOR}", linev[1]);
                    return;
                }
                Ok(addr)
            },
        };

        if let Ok(addr) = addr_result {
            unsafe {
                if OPTION.breakpoint_addr.contains(&addr) {
                    OPTION.breakpoint_addr.retain(|&address| address != addr);
                    println!("{VALID_COLOR}the breakpoint {:#x} has retain{RESET_COLOR}", addr);
                } else {
                    eprintln!("{ERR_COLOR}the breakpoint vector does not contain this address{RESET_COLOR}");
                }
            }
        }
    } else {
        eprintln!("{}", usage::USAGE_BRPT.replace("breakpoint", "retain-breakpoint"));
    }
}




