use std::num::ParseIntError;
use crate::utils::*;
use std::mem;
use std::str::FromStr;
use num::Num;
use regex::Regex;
use winapi::um::memoryapi::{VirtualProtectEx, WriteProcessMemory};
use winapi::shared::minwindef::{LPCVOID, LPVOID};
use winapi::shared::ntdef::HANDLE;
use winapi::um::winnt::{CONTEXT, PAGE_EXECUTE_READWRITE, WOW64_CONTEXT};
use crate::dbg::dbg_cmd::x64::info_reg::{ToValue, Value};
use crate::dbg::dbg_cmd::usages;
use crate::dbg::dbg_cmd::x32::info_reg::ToValue32;
use crate::utils::str_to;
use crate::dbg::RealAddr;
use crate::symbol::SYMBOLS_V;




pub fn set_memory64(process_handle: HANDLE, ctx: CONTEXT, arg: &[&str]) {
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
    target_mem(process_handle, &new_value_str, target_addr, size, &types_r);
}




pub fn target_mem(process_handle: HANDLE, value_str: &str, target_addr: u64, size: usize, types_r: &str) {
    match types_r {
        "uint8_t" | "char" | "u8" | "byte" => target_in_memory::<u8>(process_handle, &value_str, target_addr, size),
        "int8_t" | "i8"  => target_in_memory::<i8>(process_handle, value_str, target_addr, size),
        "uint16_t" | "word" | "u16" => target_in_memory::<u16>(process_handle, value_str, target_addr, size),
        "int16_t" | "i16" => target_in_memory::<i16>(process_handle, value_str, target_addr, size),
        "uint32_t" | "dword" | "u32" => target_in_memory::<u32>(process_handle, value_str, target_addr, size),
        "int32_t" | "int" | "i32" => target_in_memory::<i32>(process_handle, value_str, target_addr, size),
        "uint64_t" | "qword" | "u64" => target_in_memory::<u64>(process_handle, value_str, target_addr, size),
        "int64_t" | "i64" => target_in_memory::<i64>(process_handle, value_str, target_addr, size),
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



pub fn set_memory32(process_handle: HANDLE, ctx: WOW64_CONTEXT, linev: &[&str]) {
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
    target_mem(process_handle, &new_value_str, target_addr, size, &types_r);
}


#[derive(PartialOrd, PartialEq)]
enum Mod2 {
    IsChar,
    IsDigit,
}




pub fn target_in_memory<T: Num<FromStrRadixErr = ParseIntError> + Default + std::fmt::Debug + FromStr<Err = ParseIntError>>(process_handle: HANDLE, value_str: &str, target_addr: u64, size: usize) {
    let mut size = size;
    let mut result: Vec<T> = Vec::new();
    let mut mod_2 = Mod2::IsDigit;
    let valu_vc = value_str.chars().collect::<Vec<char>>();
    let mut i = 0;
    let mut digix_temp = Vec::new();
    while i < valu_vc.len() {
        let c = valu_vc[i];
        if mod_2 != Mod2::IsChar && (c == '\'' || c == '"') {
            mod_2 = Mod2::IsChar;
        }
        else if mod_2 == Mod2::IsChar && (c == '\'' || c == '"') {
            mod_2 = Mod2::IsDigit;
        }
        else if (c == ',' && mod_2 == Mod2::IsDigit) || i + 1 == valu_vc.len() {
            if i + 1 == valu_vc.len() && mod_2 == Mod2::IsDigit {
                digix_temp.push(c.to_string());
            }
            let digix = digix_temp.join("").replace(" ", "");
            if !digix.is_empty() {
                match str_to::<T>(&digix) {
                    Ok(value) => result.push(value),
                    Err(_) => {}
                }
            }
            digix_temp.clear();
        } else if mod_2 == Mod2::IsDigit {
            digix_temp.push(c.to_string());
        } else if mod_2 == Mod2::IsChar {
            result.push(str_to::<T>(&(c as u8).to_string()).unwrap());
        }
        i += 1;
    }

    if size == usize::MAX {
        size = result.len();
    }
    else if size > result.len() {
        result.push(T::default())
    }

    let mut addr_t = target_addr;
    let mut old_protect = 0;
    unsafe {
        if VirtualProtectEx(process_handle, addr_t as LPVOID, size, PAGE_EXECUTE_READWRITE, &mut old_protect) == 0 {
            eprintln!("{ERR_COLOR}error to remove memory protection at address {:#x}", addr_t);
            return;
        }
        for i in 0..size {
            let value_targ = &result[i];
            if WriteProcessMemory(process_handle, addr_t as LPVOID, value_targ as *const _ as LPCVOID, mem::size_of::<T>(), &mut 0) == 0 {
                eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> error when writing to memory at address : {:#x}", addr_t);
                return;
            }
            addr_t += mem::size_of::<T>() as u64;
        }
        if VirtualProtectEx(process_handle, addr_t as LPVOID, size, old_protect, &mut old_protect) == 0 {
            eprintln!("{ERR_COLOR}error to restaure memory protection at address {:#x}", addr_t);
            return;
        }
    }
    println!("{VALID_COLOR}the changes were made successfully{RESET_COLOR}")
}








