use std::num::ParseIntError;
use crate::utils::*;
use std::{io, mem, ptr};
use std::str::FromStr;
use num::Num;
use regex::Regex;
use winapi::shared::minwindef::LPVOID;
use winapi::um::memoryapi::{ReadProcessMemory, VirtualProtectEx, WriteProcessMemory};
use winapi::shared::ntdef::HANDLE;
use winapi::um::winnt::{CONTEXT, PAGE_EXECUTE_READWRITE, WOW64_CONTEXT};
use crate::dbg::dbg_cmd::x64::info_reg::{ToValue, Value};
use crate::dbg::dbg_cmd::usages;
use crate::dbg::dbg_cmd::x32::info_reg::ToValue32;
use crate::utils::str_to;
use crate::dbg::RealAddr;
use crate::pefile::NT_HEADER;
use crate::symbol::SYMBOLS_V;


fn str_t_to_all_str<T: Num>() -> Vec<String> {
    match std::any::type_name::<T>() {
        "u8" => vec!["uint8_t".to_string(), "u8".to_string(), "byte".to_string()],
        "u16" => vec!["uint16_t".to_string(), "u16".to_string(), "word".to_string()],
        "u32" => vec!["uint32_t".to_string(), "u32".to_string(), "dword".to_string()],
        "u64" => vec!["uint64_t".to_string(), "u64".to_string(), "qword".to_string()],
        "i8" => vec!["int8_t".to_string(), "i8".to_string(), "char".to_string()],
        "i16" => vec!["int16_t".to_string(), "i16".to_string()],
        "i32" => vec!["int32_t".to_string(), "i32".to_string()],
        "i64" => vec!["int64_t".to_string(), "i64".to_string()],
        _ => vec![],
    }
}


pub fn set_memory64(h_proc: HANDLE, ctx: CONTEXT, arg: &[&str]) {
    if arg.len() < 3 {
        eprintln!("{}", usages::USAGE_SET_MEM);
        return;
    }
    let types = arg[0];
    let target = arg[1];
    let new_value_str = arg[2..].join(" ");
    let mut size = 1;
    get_size(&mut size, types);
    let target_addr = match ctx.str_to_value_ctx(target) {
        Value::U64(target_addr) => target_addr,
        Value::U128(_) => {
            eprintln!("{ERR_COLOR}simd registers are not taken into account for this operation{RESET_COLOR}");
            return;
        }
        Value::Un => {
            match str_to::<u64>(target) {
                Ok(addr) => addr,
                Err(_) => unsafe {
                    if let Some(sym) = SYMBOLS_V.symbol_file.iter().find(|s|s.name == target) {
                        sym.real_addr64(ctx)
                    }else {
                        eprintln!("{ERR_COLOR}invalid target : '{target}'{RESET_COLOR}");
                        return;
                    }
                }
            }
        }
    };

    let types_r = types.split('[').next().unwrap_or_default().to_lowercase();
    target_mem(h_proc, &new_value_str, target_addr, size, &types_r);
}




pub fn target_mem(h_proc: HANDLE, value_str: &str, target_addr: u64, size: usize, types_r: &str) {
    match types_r {
        "uint8_t" | "u8" | "byte" => target_in_memory::<u8>(h_proc, &value_str, target_addr, size),
        "int8_t" | "i8" | "char"  => target_in_memory::<i8>(h_proc, value_str, target_addr, size),
        "uint16_t" | "word" | "u16" => target_in_memory::<u16>(h_proc, value_str, target_addr, size),
        "int16_t" | "i16" => target_in_memory::<i16>(h_proc, value_str, target_addr, size),
        "uint32_t" | "dword" | "u32" => target_in_memory::<u32>(h_proc, value_str, target_addr, size),
        "int32_t" | "int" | "i32" => target_in_memory::<i32>(h_proc, value_str, target_addr, size),
        "uint64_t" | "qword" | "u64" => target_in_memory::<u64>(h_proc, value_str, target_addr, size),
        "int64_t" | "i64" => target_in_memory::<i64>(h_proc, value_str, target_addr, size),
        _ => eprintln!("{ERR_COLOR}unsupported type{RESET_COLOR}"),
    }
}






fn get_size(size: &mut usize, type_t: &str) {
    let re = Regex::new(r"\[(.*?)]").unwrap();
    for cap in re.captures_iter(type_t) {
        if let Some(numd) = cap.get(1) {
            match str_to::<usize>(numd.as_str()) {
                Ok(num) => *size = num,
                Err(e) => if e.to_string().contains("empty string") {
                    *size = usize::MAX;
                }
            }
        }
    }
}



pub fn set_memory32(h_proc: HANDLE, ctx: WOW64_CONTEXT, linev: &[&str]) {
    if linev.len() < 3 {
        eprintln!("{}", usages::USAGE_SET_MEM);
        return;
    }
    let types = linev[0];
    let target = linev[1];
    let new_value_str = linev[2..].join(" ");
    let mut size = 1;
    get_size(&mut size, types);
    let mut target_addr = ctx.str_to_ctx(target) as u64;
    if target_addr == 0 {
        match str_to::<u32>(target) {
            Ok(addr) => target_addr = addr as u64,
            Err(e) =>  {
                eprintln!("{ERR_COLOR}invalid target address : {e}{RESET_COLOR}");
                return;
            }
        }
    }

    let types_r = types.split('[').next().unwrap_or_default().to_lowercase();
    target_mem(h_proc, &new_value_str, target_addr, size, &types_r);
}




pub fn target_in_memory<T: Num<FromStrRadixErr = ParseIntError> + Default + std::fmt::Debug + FromStr<Err = ParseIntError> + Clone>(h_proc: HANDLE, value_str: &str, target_addr: u64, size: usize)  {
    let mut result: Vec<T> = Vec::new();
    let deref_p = Regex::new(r"(\*+)\(([^*\[\]]+)(?:\[(\d*)])?(\*+)\)(0x[0-9a-fA-F]+)").unwrap();
    let v_part: Vec<&str> = value_str.split(',').map(|s| s.trim()).collect();
    let type_a = str_t_to_all_str::<T>();
    for w in v_part {
        if (w.starts_with("'") && w.ends_with("'")) || (w.starts_with("\"") && w.ends_with("\"")) {
            let w_trimmed = &w[1..w.len() - 1];
            for c in w_trimmed.chars() {
                match str_to::<T>(&(c as u8).to_string()) {
                    Ok(v) => result.push(v),
                    Err(e) => eprintln!("{ERR_COLOR}Invalid value: {} : {e}{RESET_COLOR}", c),
                }
            }
        }
        else if let Some(caps) = deref_p.captures(value_str) {
            let first_ast = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let type_str = caps.get(2).map(|m| m.as_str()).unwrap_or("");
            let count = caps.get(3).and_then(|m| str_to::<usize>(m.as_str()).ok()).unwrap_or(1);
            let ast2 = caps.get(4).map_or("", |m| m.as_str());
            if first_ast.len() != ast2.len() {
                eprintln!("{ERR_COLOR}invalid type: deref count : {first_ast} - ptr type : {ast2}{RESET_COLOR}");
                continue
            }
            let addr_str = caps.get(5).map(|m| m.as_str()).unwrap_or("");
            if type_a.contains(&type_str.to_string()) {
                match str_to::<u64>(addr_str) {
                    Ok(mut addr) => unsafe {
                        for _ in 0..first_ast.len() - 1 {
                            if ReadProcessMemory(h_proc, addr as LPVOID, ptr::addr_of_mut!(addr) as LPVOID, NT_HEADER.unwrap().get_size_of_arch(), &mut 0) == 0 {
                                eprintln!("{ERR_COLOR}Bad ptr: {:#x} : {}{RESET_COLOR}", addr, io::Error::last_os_error());
                                continue
                            }
                        }
                        let mut value_t = vec![T::default(); count];
                        if ReadProcessMemory(h_proc, addr as LPVOID, value_t.as_mut_ptr() as LPVOID, mem::size_of::<T>() * count, &mut 0) == 0  {
                            eprintln!("{ERR_COLOR}Error dereferencing memory at address: {:#x} : {}{RESET_COLOR}", addr, io::Error::last_os_error());
                            return;
                        }
                        result.extend(value_t);
                    }
                    Err(e) => eprintln!("{ERR_COLOR}Invalid address: {} : {e}{RESET_COLOR}", addr_str),
                }
            } else {
                eprintln!("{ERR_COLOR}Type mismatch: expected one of {:?}, found {}{RESET_COLOR}", type_a, type_str);
            }
        }
        else {
            match str_to::<T>(w) {
                Ok(v) => result.push(v),
                Err(e) => eprintln!("{ERR_COLOR}Invalid value: {w} : {e}{RESET_COLOR}"),
            }
        }
    }


    let ef_size = size.min(result.len()) * mem::size_of::<T>();
    let mut old_protect = 0;

    unsafe {
        if VirtualProtectEx(h_proc, target_addr as LPVOID, ef_size, PAGE_EXECUTE_READWRITE, &mut old_protect) == 0 {
            eprintln!("{ERR_COLOR}Error removing memory protection at address {:#x} : {}{RESET_COLOR}", target_addr, io::Error::last_os_error());
            return;
        }

        if WriteProcessMemory(h_proc, target_addr as LPVOID, result.as_ptr() as LPVOID, ef_size, &mut 0) == 0 {
            eprintln!("{ERR_COLOR}Error writing to memory at address: {:#x}: {}{RESET_COLOR}", target_addr, io::Error::last_os_error());
            return;
        }

        if VirtualProtectEx(h_proc, target_addr as LPVOID, ef_size, old_protect, &mut old_protect) == 0 {
            eprintln!("{ERR_COLOR}Error restoring memory protection at address {:#x}: {}{RESET_COLOR}", target_addr, io::Error::last_os_error());
            return;
        }
    }
    println!("{VALID_COLOR}The changes were made successfully{RESET_COLOR}");
}








