use std::{io, ptr};
use winapi::shared::minwindef::LPVOID;
use winapi::um::memoryapi::{ReadProcessMemory, WriteProcessMemory};
use winapi::um::winnt::{CONTEXT, HANDLE};
use crate::utils::*;
use crate::{OPTION, usage};
use crate::command::{hook, skip};
use crate::command::stret::BREAK_RET;
use crate::dbg::{BASE_ADDR, memory, SAVEINSN};
use crate::dbg::memory::watchpoint;
use crate::pefile::function::FUNC_INFO;
use crate::symbol::SYMBOLS_V;

pub fn remove_element(linev: &[&str]){
    if linev.len() != 3 {
        println!("{}", usage::USAGE_REMOVE);
        return;
    }
    let element = linev[1];
    let target = linev[2];
    let addr = match str_to::<i64>(target) {
        Ok(value) => value,
        Err(_) => unsafe {
            if let Some(sym) = SYMBOLS_V.symbol_file.iter().find(|s|s.name == target) {
                sym.offset
            }else if !crate::pefile::function::CR_FUNCTION.iter().any(|cf|cf.name == target){
                eprintln!("{ERR_COLOR}invalid target : {target}{RESET_COLOR}");
                return;
            }else {
                0
            }
        }
    };
    let binding = element.to_lowercase();
    let element = binding.as_str();

    let vec_option = unsafe {
        match element {
            "breakpoint" | "b" => Some(ptr::addr_of_mut!(OPTION.breakpoint_addr)),
            "break-ret" | "b-ret" => Some(ptr::addr_of_mut!(BREAK_RET)),
            "skip" => Some(ptr::addr_of_mut!(skip::SKIP_ADDR)),
            "hook" => {
                hook::HOOK_FUNC.retain(|h|h.target != addr as u64);
                println!("{VALID_COLOR}{target} has been retained successfully{RESET_COLOR}");
                return;
            }
            "watchpoint" | "watch" | "w" => {
                OPTION.watchpts.retain(|w|w.offset != addr);
                println!("{VALID_COLOR}{target} has been retained successfully{RESET_COLOR}");
                return;
            }
            "crt-func" | "create-function" => {
                crate::pefile::function::CR_FUNCTION.retain(|cf|cf.name != target);
                println!("{VALID_COLOR}{target} has been retained successfully{RESET_COLOR}");
                return;
            }
            _ => None,
        }
    };

    unsafe {
        if let Some(vec_ptr) = vec_option {
            let vec = &mut *vec_ptr;
            vec.retain(|&a| a != addr as u64);
            println!("{VALID_COLOR}{target} has been retained successfully{RESET_COLOR}");
        } else {
            println!("{ERR_COLOR}'{element}' is not a valid target{RESET_COLOR}");
        }
    }
}









pub fn remove_element_proc(linev: &[&str], process_handle: HANDLE, ctx: &mut CONTEXT){
    if linev.len() != 3 {
        println!("{}", usage::USAGE_REMOVE);
        return;
    }
    let element = linev[1];
    let target = linev[2];
    let addr = match str_to::<u64>(target) {
        Ok(value) => value,
        Err(_) => unsafe {
            if let Some(sym) = SYMBOLS_V.symbol_file.iter().find(|s|s.name == target) {
                if sym.offset < 0 {
                    eprintln!("{ERR_COLOR}the address must not be negative{RESET_COLOR}");
                    return;
                }
                sym.offset as u64
            }else {
                eprintln!("{ERR_COLOR}invalid target : {target}{RESET_COLOR}");
                return;
            }
        }
    };

    match element {
        "breakpoint" | "b" => unsafe {
            OPTION.breakpoint_addr.retain(|&br|br != addr);
            memory::breakpoint::restore_byte_of_brkpt(process_handle, addr + BASE_ADDR);
        }
        "break-ret" | "b-ret" => unsafe {
            if let Some(func) = FUNC_INFO.iter().find(|f|f.BeginAddress == addr as u32) {
                for br in &*ptr::addr_of!(OPTION.breakpoint_addr) {
                    let br = *br;
                    if br >= func.BeginAddress as u64 && br <= func.EndAddress as u64{
                        let va_br = br + BASE_ADDR;
                        let mut byte = 0u8;
                        if ReadProcessMemory(process_handle, va_br as LPVOID, &mut byte as *mut _ as LPVOID, 1, &mut 0) == 0 {
                            eprintln!("{ERR_COLOR}failed to read memory at address {:#x} : {}", va_br, io::Error::last_os_error());
                            return;
                        }
                        if byte == 0xcc {
                            if let Some(insn_save) = SAVEINSN.iter().find(|is|is.addr == br + BASE_ADDR && is.last_oc == 0xc3) {
                                OPTION.breakpoint_addr.retain(|&b|b != br);
                                BREAK_RET.retain(|&b|b != br);
                                if WriteProcessMemory(process_handle, insn_save.addr as LPVOID, &insn_save.last_oc as *const _ as LPVOID, 1, &mut 0) == 0 {
                                    eprintln!("{ERR_COLOR}failed to write memory at address {:#x} : {}", insn_save.addr, io::Error::last_os_error());
                                    return;
                                }
                                println!("{VALID_COLOR}breakpoint at address {:#x} has been removed", va_br);
                                return;
                            }
                        }
                    }
                }
                eprintln!("{ERR_COLOR}target not found{RESET_COLOR}");
            }else {
                eprintln!("{ERR_COLOR}the address you specified is not a function{RESET_COLOR}");
            }
        }
        "watchpoint" | "watch" | "w" => unsafe {
            if let Some(pos) = OPTION.watchpts.iter().position(|w|w.offset == addr as i64) {
                watchpoint::clear_dreg(ctx, pos);
                OPTION.watchpts.remove(pos);
                println!("{VALID_COLOR}watchpoint has been deleted successfully{RESET_COLOR}");
            }else {
                eprintln!("{ERR_COLOR}the watchpoint for address {:#x} is not found", addr);
            }
        }
        "skip" => unsafe {
            if skip::SKIP_ADDR.contains(&(addr)) {
                skip::SKIP_ADDR.retain(|&saddr|saddr != addr);
                if let Some(insn_save) = SAVEINSN.iter().find(|s|s.addr == addr + BASE_ADDR) {
                    if WriteProcessMemory(process_handle, insn_save.addr as LPVOID, &insn_save.last_oc as *const _ as LPVOID, 1, &mut 0) == 0 {
                        eprintln!("{ERR_COLOR}failed to write memory at address {:#x} : {}", insn_save.addr, io::Error::last_os_error());
                        return;
                    }else {
                        println!("{VALID_COLOR}the function will now run as before{RESET_COLOR}");
                    }
                }
            }
        }
        "hook" => unsafe {
            hook::HOOK_FUNC.retain(|h|h.target != addr);
            if let Some(insn_save) = SAVEINSN.iter().find(|s|s.addr == addr + BASE_ADDR) {
                if WriteProcessMemory(process_handle, insn_save.addr as LPVOID, &insn_save.last_oc as *const _ as LPVOID, 1, &mut 0) == 0 {
                    eprintln!("{ERR_COLOR}failed to write memory at address {:#x} : {}", insn_save.addr, io::Error::last_os_error());
                    return;
                }else {
                    println!("{VALID_COLOR}the function will now run as before{RESET_COLOR}");
                    OPTION.breakpoint_addr.retain(|br|br != &addr);
                }
            }
        }
        _ => {}
    }
}