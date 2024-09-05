use std::{io, mem};
use regex::Regex;
use winapi::shared::minwindef::LPVOID;
use winapi::shared::ntdef::HANDLE;
use winapi::um::memoryapi::ReadProcessMemory;
use winapi::um::winnt::CONTEXT;
use crate::dbg::dbg_cmd::x64::info_reg::{ToValue, Value};
use crate::dbg::dbg_cmd::usages;
use crate::dbg::RealAddr;
use crate::utils::*;
use crate::symbol::SYMBOLS_V;

unsafe fn read_memory<T: std::fmt::LowerHex + Default + Clone>(process_handle: HANDLE, address: usize, size: usize, bytes_read: &mut usize){
    let r_size = size * mem::size_of::<T>();
    let mut result = vec![T::default(); size];
    if ReadProcessMemory(process_handle, address as LPVOID, result.as_mut_ptr() as LPVOID, r_size, bytes_read) == 0 {
        eprintln!("{ERR_COLOR}failed to read memory : {}{RESET_COLOR}", io::Error::last_os_error());
        return;
    }
    print!("{:#x}: {}{VALUE_COLOR}",address, if size > 1 {"["}else {""});
    for (i, elm) in result.iter().enumerate() {
        print!("{:#x}{}", elm, if size > 1 || i != size - 1 {", "} else {""});
    }
    println!("{RESET_COLOR}{}", if size > 1 {"]"}else {""});
}


unsafe fn read_memory_flt<T: num::Float + std::fmt::Display + Default>(process_handle: HANDLE, address: usize, size: usize, byte_read: &mut usize) {
    let r_size = size * mem::size_of::<T>();
    print!("{:#x}: {}{VALUE_COLOR}",address, if size > 1 {"["}else {""});
    let mut result = vec![T::default(); size];
    if ReadProcessMemory(process_handle, address as LPVOID, result.as_mut_ptr() as LPVOID, r_size, byte_read) == 0 {
        eprintln!("{ERR_COLOR}failed to read memory : {}", io::Error::last_os_error());
        return;
    }
    for (i, elm) in result.iter().enumerate(){
        print!("{}{}", elm, if size > 1 || i != size - 1 {", "} else {""});
    }
    println!("{RESET_COLOR}{}", if size > 1 {"]"}else {""});
}


pub fn handle_deref(linev: &[&str], ctx: CONTEXT, process_handle: HANDLE) {
    if linev.len() < 3 {
        eprintln!("{}", usages::USAGE_DEREF);
        return
    }
    let dtype = linev[1];
    let target = linev[2];
    if target == "" {
        eprintln!("{ERR_COLOR}empty target{RESET_COLOR}");
        return;
    }
    let address = if let Ok(addr) = str_to::<u64>(target) {
        addr
    } else {
        let value = ctx.str_to_value_ctx(target);
        let addr = match value {
            Value::U64(value) => value,
            Value::U128(xmm) => xmm.Low,
            Value::Un => 0,
        };
        if addr != 0 {
            addr
        }
        else {
            unsafe {
                if let Some(sym) = SYMBOLS_V.symbol_file.iter().find(|s| s.name == target) {
                    sym.real_addr64(ctx)
                } else {
                    eprintln!("{ERR_COLOR}invalid target : '{target}'{RESET_COLOR}");
                    return;
                }
            }
        }
    };
    if let Err(err) = deref_memory(process_handle, dtype, address as usize) { eprintln!("{ERR_COLOR}{}{RESET_COLOR}", err) }
}



pub fn deref_memory(process_handle: HANDLE, dtype: &str, address: usize) -> Result<(), String> {
    let mut bytes_read = 0;
    let re = Regex::new(r"\[(.*?)]").unwrap();
    let mut size = 1;
    if dtype != "char[]" {
        for cap in re.captures_iter(dtype) {
            if let Some(numd) = cap.get(1) {
                match str_to::<usize>(numd.as_str()) {
                    Ok(num) => size = num,
                    Err(e) => return Err(e.to_string())
                }
            }
        }
    }else {
        size = 0;
    }

    let types_r = dtype.split('[').next().unwrap_or_default();

    unsafe {
        match types_r {
            "uint8_t" | "byte" => read_memory::<u8>(process_handle, address, size, &mut bytes_read),
            "int8_t" => read_memory::<i8>(process_handle, address, size, &mut bytes_read),
            "uint16_t" | "word" => read_memory::<u16>(process_handle, address, size, &mut bytes_read),
            "int16_t" | "short" => read_memory::<i16>(process_handle, address, size, &mut bytes_read),
            "uint32_t" | "u32" => read_memory::<u32>(process_handle, address, size, &mut bytes_read),
            "int32_t" | "int" | "long" => read_memory::<i32>(process_handle, address, size, &mut bytes_read),
            "uint64_t" | "qword" => read_memory::<u64>(process_handle, address, size, &mut bytes_read),
            "int64_t" => read_memory::<i64>(process_handle, address, size, &mut bytes_read),
            "float" | "f32" => read_memory_flt::<f32>(process_handle, address, size, &mut bytes_read),
            "double" | "f64" => read_memory_flt::<f64>(process_handle, address, size, &mut bytes_read),
            "char" => read_string(process_handle, address, size,  &mut bytes_read),
            _ => return Err(format!("Unknown type: {}", dtype)),
        }
    }
    Ok(())
}






pub fn espc(input: &[u8]) -> String {
    let mut result = String::new();
    for &byte in input {
        let escaped = std::ascii::escape_default(byte);
        let escaped_char = escaped.map(|b| b as char).collect::<String>();
        if escaped_char.len() > 1 || escaped_char.chars().next().unwrap().is_control() {
            result.push_str("\x1B[33m");
            result.push_str(&escaped_char);
            result.push_str("\x1B[0m");
        } else {
            result.push_str("\x1B[32m");
            result.push(escaped_char.chars().next().unwrap());
            result.push_str("\x1B[0m");
        }
    }
    result
}



pub unsafe fn read_string(process_handle: HANDLE, address: usize, size: usize, bytes_read: &mut usize) {
    let mut b_str = Vec::new();
    let addr_s = address;
    let mut i = 0;
    let mut b = 1u8;
    loop {
        if ReadProcessMemory(process_handle, (addr_s + i) as LPVOID, &mut b as *mut _ as LPVOID, 1, &mut 0) == 0 {
            eprintln!("{ERR_COLOR}failed to read memory at address {:#x} : {}", address, io::Error::last_os_error());
            return;
        }
        if size != 0 && i == size || size == 0 && b == 0 {
            break
        }
        b_str.push(b);
        *bytes_read += 1;
        i+=1;
    }
    let str_byte = &b_str[..if size != 0 {size} else {b_str.len()}];
    let str_r = espc(str_byte);
    println!("{:#x}: \"{}\"", address, str_r);
}
