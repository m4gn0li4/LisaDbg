use crate::dbg::memory::watchpoint;
use crate::dbg::{memory, RealAddr, BASE_ADDR};
use crate::symbol::SYMBOLS_V;
use crate::utils::*;
use crate::{usage, ALL_ELM};
use std::ptr;
use winapi::um::winnt::{CONTEXT, HANDLE};

pub fn remove_element(linev: &[&str]) {
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
            if let Some(sym) = SYMBOLS_V.symbol_file.iter().find(|s| s.name == target) {
                sym.offset
            } else {
                0
            }
        },
    };

    let vec_option = unsafe {
        match element {
            "breakpoint" | "b" => Some(ptr::addr_of_mut!(ALL_ELM.break_rva)),
            "break-ret" | "b-ret" => Some(ptr::addr_of_mut!(ALL_ELM.break_ret)),
            "skip" => Some(ptr::addr_of_mut!(ALL_ELM.skip_addr)),
            "break-va" | "b-va" => Some(ptr::addr_of_mut!(ALL_ELM.break_va)),
            "break-ret-va" => Some(ptr::addr_of_mut!(ALL_ELM.break_ret_va)),
            "hook" => {
                if addr == 0 {
                    eprintln!("{ERR_COLOR}invalid target: {target}{RESET_COLOR}");
                    return;
                }
                ALL_ELM.hook.retain(|h| h.target != addr as u64);
                println!("{VALID_COLOR}{target} has been retained successfully{RESET_COLOR}");
                return;
            }
            "watchpoint" | "watch" | "w" => {
                if addr == 0 {
                    eprintln!("{ERR_COLOR}invalid target: {target}{RESET_COLOR}");
                    return;
                }
                ALL_ELM.watchpts.retain(|w| w.offset != addr);
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
                    "func" | "function" => ALL_ELM.crt_func.retain(|f| f.name != target_name),
                    "struct" => ALL_ELM
                        .struct_def
                        .retain(|s| s.get_name_of_struct() != target_name),
                    _ => {
                        eprintln!("{ERR_COLOR}unknow element '{target}'{RESET_COLOR}");
                        return;
                    }
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
            if let Some(pos) = vec.iter().position(|&e|e == addr as u64) {
                vec.remove(pos);
                println!("{VALID_COLOR}{target} has been retained successfully{RESET_COLOR}");
            }else {
                println!("{ERR_COLOR}'{element}' is not a valid target{RESET_COLOR}");
            }
        } else {
            println!("{ERR_COLOR}'{element}' is not a valid target{RESET_COLOR}");
        }
    }
}



pub fn remove_element_proc(linev: &[&str], h_proc: HANDLE, ctx: &mut CONTEXT) {
    if linev.len() != 3 {
        println!("{}", usage::USAGE_REMOVE);
        return;
    }
    let element = linev[1];
    let target = linev[2];
    let addr = match str_to::<u64>(target) {
        Ok(value) => value,
        Err(_) => unsafe {
            if let Some(sym) = SYMBOLS_V.symbol_file.iter().find(|s| s.name == target) {
                sym.real_addr64(*ctx)
            } else {
                eprintln!("{ERR_COLOR}invalid target : {target}{RESET_COLOR}");
                return;
            }
        },
    };

    match element {
        "breakpoint" | "b" => unsafe {
            if let Some(pos) = ALL_ELM.break_rva.iter().position(|&b|b == addr) {
                memory::breakpoint::restore_byte_of_brkpt(h_proc, ALL_ELM.break_rva.remove(pos) + BASE_ADDR);
            }else {
                eprintln!("{ERR_COLOR}no breakpoint was set for this rva : {:#x}{RESET_COLOR}", addr);
            }
        },
        "break-va" | "b-va" => unsafe {
            if let Some(pos) = ALL_ELM.break_va.iter().position(|&b|b == addr) {
                memory::breakpoint::restore_byte_of_brkpt(h_proc, ALL_ELM.break_va.remove(pos) + BASE_ADDR);
            }else {
                eprintln!("{ERR_COLOR}no breakpoint was set for this va : {:#x}{RESET_COLOR}", addr);
            }
        },
        "break-ret" | "b-ret" => unsafe {
            if let Some(pos) = ALL_ELM.break_ret.iter().position(|&b|b == addr) {
                memory::breakpoint::restore_byte_of_brkpt(h_proc, ALL_ELM.break_ret.remove(pos) + BASE_ADDR);
                eprintln!("{WAR_COLOR}if a breakpoint is already set on the return address, you will need to remove it manually (it's a b-va breakpoint){RESET_COLOR}");
            }else {
                eprintln!("{ERR_COLOR}no b-ret was set for this addr : {:#x}{RESET_COLOR}", addr);
            }
        },
        "watchpoint" | "watch" | "w" => unsafe {
            if let Some(pos) = ALL_ELM.watchpts.iter().position(|w| w.real_addr64(*ctx) == addr) {
                watchpoint::clear_dreg(ctx, pos);
                ALL_ELM.watchpts.remove(pos);
                println!("{VALID_COLOR}watchpoint has been deleted successfully{RESET_COLOR}");
            } else {
                eprintln!("{ERR_COLOR}the watchpoint for address {:#x} is not found", addr);
            }
        },
        "skip" => unsafe {
            if let Some(pos) = ALL_ELM.skip_addr.iter().position(|&a| a == addr) {
                memory::breakpoint::restore_byte_of_brkpt(h_proc, ALL_ELM.skip_addr.remove(pos) + BASE_ADDR);
            }else {
                eprintln!("{ERR_COLOR}no skip has been defined for this function: {target}{RESET_COLOR}");
            }
        },
        "hook" => unsafe {
            if let Some(pos) = ALL_ELM.hook.iter().position(|h| h.target == addr) {
                ALL_ELM.hook.remove(pos);
                memory::breakpoint::restore_byte_of_brkpt(h_proc, addr + BASE_ADDR);
            }else {
                eprintln!("{ERR_COLOR}invalid target : {target}{RESET_COLOR}");
            }
        },
        "def" => unsafe {
            if linev.len() != 4 {
                eprintln!("{ERR_COLOR}Please specify a target for remove{RESET_COLOR}");
                return;
            }
            let target_name = linev[3].to_string();
            match target {
                "func" | "function" => ALL_ELM.crt_func.retain(|f| f.name != target_name),
                "struct" => ALL_ELM.struct_def.retain(|s| s.get_name_of_struct() != target_name),
                _ => {
                    eprintln!("{ERR_COLOR}unknow element '{target}'{RESET_COLOR}");
                    return;
                }
            }
            println!("{VALID_COLOR}{target_name} was retain with successfully{RESET_COLOR}");
            return;
        },
        _ => {}
    }
}
