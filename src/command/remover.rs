use std::{io, ptr};
use winapi::shared::minwindef::LPVOID;
use winapi::um::memoryapi::WriteProcessMemory;
use winapi::um::winnt::{CONTEXT, HANDLE};
use crate::utils::*;
use crate::{usage, ALL_ELM};
use crate::dbg::{BASE_ADDR, memory, SAVEINSN};
use crate::dbg::memory::watchpoint;
use crate::symbol::SYMBOLS_V;



pub fn remove_element(linev: &[&str]){
    if linev.len() < 3 {
        println!("{}", usage::USAGE_REMOVE);
        return;
    }
    let element = linev[1];
    let target = linev[2];
    let binding = element.to_lowercase();
    let element = binding.as_str();
    let addr = match str_to::<i64>(target) {
        Ok(value) => value,
        Err(_) => unsafe {
            if let Some(sym) = SYMBOLS_V.symbol_file.iter().find(|s|s.name == target) {
                sym.offset
            }else {
                0
            }
        }
    };

    let vec_option = unsafe {
        match element {
            "breakpoint" | "b" => Some(ptr::addr_of_mut!(ALL_ELM.break_rva)),
            "break-ret" | "b-ret" => Some(ptr::addr_of_mut!(ALL_ELM.break_ret)),
            "skip" => Some(ptr::addr_of_mut!(ALL_ELM.skip_addr)),
            "hook" => {
                if addr == 0 {
                    eprintln!("{ERR_COLOR}invalid target: {target}{RESET_COLOR}");
                    return;
                }
                ALL_ELM.hook.retain(|h|h.target != addr as u64);
                println!("{VALID_COLOR}{target} has been retained successfully{RESET_COLOR}");
                return;
            }
            "watchpoint" | "watch" | "w" => {
                if addr == 0 {
                    eprintln!("{ERR_COLOR}invalid target: {target}{RESET_COLOR}");
                    return;
                }
                ALL_ELM.watchpts.retain(|w|w.offset != addr);
                println!("{VALID_COLOR}{target} has been retained successfully{RESET_COLOR}");
                return;
            }
            "def" => {
                if linev.len() != 4 {
                    eprintln!("{ERR_COLOR}Please specify a target for remove{RESET_COLOR}");
                    return;
                }
                let target_name = linev[3].to_string();
                match target {
                    "func" | "function" => ALL_ELM.crt_func.retain(|f|f.name != target_name),
                    "struct" => ALL_ELM.struct_def.retain(|s|s.get_name_of_struct() != target_name),
                    _ => {
                        eprintln!("{ERR_COLOR}unknow element '{target}'{RESET_COLOR}");
                        return;
                    },
                }
                println!("{VALID_COLOR}{target_name} was retain with successfully{RESET_COLOR}");
                return;
            }
            _ => None,
        }
    };

    unsafe {
        if let Some(vec_ptr) = vec_option {
            if addr == 0 {
                eprintln!("{ERR_COLOR}invalid target: {target}{RESET_COLOR}");
                return;
            }
            let vec = &mut *vec_ptr;
            vec.retain(|&a| a != addr as u64);
            println!("{VALID_COLOR}{target} has been retained successfully{RESET_COLOR}");
        } else {
            println!("{ERR_COLOR}'{element}' is not a valid target{RESET_COLOR}");
        }
    }
}









pub fn remove_element_proc(linev: &[&str], h_proc: HANDLE, ctx: &mut CONTEXT){
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
            ALL_ELM.break_rva.retain(|&br|br != addr);
            memory::breakpoint::restore_byte_of_brkpt(h_proc, addr + BASE_ADDR);
        }
        "break-va" | "b-va" => unsafe {
            ALL_ELM.break_va.retain(|&br|br != addr);
            memory::breakpoint::restore_byte_of_brkpt(h_proc, addr);
        }
        "break-ret" | "b-ret" => unsafe {
            let mut find = false;
            ALL_ELM.break_ret.retain(|&b_ret|
                return if b_ret == addr {
                    memory::breakpoint::restore_byte_of_brkpt(h_proc, addr + BASE_ADDR);
                    println!("{VALID_COLOR}{:#x} has retain with successfully{RESET_COLOR}", addr);
                    find = true;
                    false
                } else {
                    true
                }
            );
            if !find {
                eprintln!("{ERR_COLOR}invalid target : {target}{RESET_COLOR}");
            }
        }
        "watchpoint" | "watch" | "w" => unsafe {
            if let Some(pos) = ALL_ELM.watchpts.iter().position(|w|w.offset == addr as i64) {
                watchpoint::clear_dreg(ctx, pos);
                ALL_ELM.watchpts.remove(pos);
                println!("{VALID_COLOR}watchpoint has been deleted successfully{RESET_COLOR}");
            }else {
                eprintln!("{ERR_COLOR}the watchpoint for address {:#x} is not found", addr);
            }
        }
        "skip" => unsafe {
            if ALL_ELM.skip_addr.contains(&(addr)) {
                ALL_ELM.skip_addr.retain(|&saddr|saddr != addr);
                if let Some(insn_save) = SAVEINSN.iter().find(|s|s.addr == addr + BASE_ADDR) {
                    if WriteProcessMemory(h_proc, insn_save.addr as LPVOID, &insn_save.last_oc as *const _ as LPVOID, 1, &mut 0) == 0 {
                        eprintln!("{ERR_COLOR}failed to write memory at address {:#x} : {}", insn_save.addr, io::Error::last_os_error());
                        return;
                    }else {
                        println!("{VALID_COLOR}the function will now run as before{RESET_COLOR}");
                    }
                }
            }
        }
        "hook" => unsafe {
            ALL_ELM.hook.retain(|h|h.target != addr);
            if let Some(insn_save) = SAVEINSN.iter().find(|s|s.addr == addr + BASE_ADDR) {
                if WriteProcessMemory(h_proc, insn_save.addr as LPVOID, &insn_save.last_oc as *const _ as LPVOID, 1, &mut 0) == 0 {
                    eprintln!("{ERR_COLOR}failed to write memory at address {:#x} : {}", insn_save.addr, io::Error::last_os_error());
                    return;
                }else {
                    println!("{VALID_COLOR}the function will now run as before{RESET_COLOR}");
                    ALL_ELM.break_rva.retain(|br|br != &addr);
                }
            }
        }
        "def" => unsafe {
            if linev.len() != 4 {
                eprintln!("{ERR_COLOR}Please specify a target for remove{RESET_COLOR}");
                return;
            }
            let target_name = linev[3].to_string();
            match target {
                "func" | "function" => ALL_ELM.crt_func.retain(|f|f.name != target_name),
                "struct" => ALL_ELM.struct_def.retain(|s|s.get_name_of_struct() != target_name),
                _ => {
                    eprintln!("{ERR_COLOR}unknow element '{target}'{RESET_COLOR}");
                    return;
                },
            }
            println!("{VALID_COLOR}{target_name} was retain with successfully{RESET_COLOR}");
            return;
        }
        _ => {}
    }
}