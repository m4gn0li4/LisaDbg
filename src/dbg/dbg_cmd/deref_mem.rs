use std::{io, mem};
use regex::Regex;
use winapi::shared::minwindef::{LPCVOID, LPVOID};
use winapi::shared::ntdef::HANDLE;
use winapi::um::memoryapi::ReadProcessMemory;
use winapi::um::winnt::CONTEXT;
use crate::dbg::dbg_cmd::usages;
use crate::log::*;

pub unsafe fn read_memory<T: std::fmt::LowerHex>(process_handle: HANDLE, address: usize, size: usize, bytes_read: &mut usize) {
    let mut addr_tr = address;
    for _ in 0..size {
        let mut buffer: T = mem::zeroed();
        if ReadProcessMemory(process_handle, addr_tr as LPCVOID, &mut buffer as *mut _ as LPVOID, mem::size_of::<T>(), bytes_read) == 0 {
            eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to read memory: {}", io::Error::last_os_error());
            break
        } else {
            println!("{:#x}: {VALUE_COLOR}{:#x}{RESET_COLOR}", addr_tr, buffer);
        }
        addr_tr += mem::size_of::<T>()
    }
}


pub fn handle_deref(linev: &[&str], ctx: CONTEXT, process_handle: HANDLE) {
    if linev.len() < 3 {
        eprintln!("{}", usages::USAGE_DEREF);
        return
    }
    let dtype = linev[1];
    let target = linev[2];
    let address = if let Ok(addr) = str_to::<usize>(target) {
        addr
    } else {
        get_value_with_reg(target, ctx) as usize
    };
    if address == 0 { return }
    if let Err(err) = deref_memory(process_handle, dtype, address) { eprintln!("{ERR_COLOR}{}{RESET_COLOR}", err) }
}


fn get_value_with_reg(reg: &str, ctx: CONTEXT) -> u64{
    match reg {
        "rsp" => ctx.Rsp,
        "rbp" => ctx.Rbp,
        "rax" => ctx.Rax,
        "rbx" => ctx.Rbx,
        "rcx" => ctx.Rcx,
        "rdx" => ctx.Rdx,
        "rsi" => ctx.Rsi,
        "rdi" => ctx.Rdi,
        "rip" => ctx.Rip,
        "r8"  => ctx.R8,
        "r9"  => ctx.R9,
        "r10" => ctx.R10,
        "r11" => ctx.R11,
        "r12" => ctx.R12,
        "r13" => ctx.R13,
        "r14" => ctx.R14,
        "r15" => ctx.R15,
        _ => {
            eprintln!("{ERR_COLOR}Invalid address or register: {reg}{RESET_COLOR}");
            return 0
        }
    }
}



fn deref_memory(process_handle: HANDLE, dtype: &str, address: usize) -> Result<(), String> {
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
    }
    let mut types_r = dtype;
    if types_r != "char[]" {
        types_r = dtype.split('[').next().unwrap_or_default();
    }

    unsafe {
        match types_r {
            "uint8_t" | "char" => read_memory::<u8>(process_handle, address, size, &mut bytes_read),
            "int8_t" => read_memory::<i8>(process_handle, address, size, &mut bytes_read),
            "uint16_t" => read_memory::<u16>(process_handle, address, size, &mut bytes_read),
            "int16_t" => read_memory::<i16>(process_handle, address, size, &mut bytes_read),
            "uint32_t" => read_memory::<u32>(process_handle, address, size, &mut bytes_read),
            "int32_t" => read_memory::<i32>(process_handle, address, size, &mut bytes_read),
            "uint64_t" => read_memory::<u64>(process_handle, address, size, &mut bytes_read),
            "int64_t" => read_memory::<i64>(process_handle, address, size, &mut bytes_read),
            "char[]" => read_string(process_handle, address, &mut bytes_read),
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



pub unsafe fn read_string(process_handle: HANDLE, address: usize, bytes_read: &mut usize) {
    let mut buffer = vec![0u8; 256];
    if ReadProcessMemory(process_handle, address as LPCVOID, buffer.as_mut_ptr() as LPVOID, buffer.len(), bytes_read) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to read memory: {}", io::Error::last_os_error());
        return;
    }
    if *bytes_read < buffer.len() {
        buffer[*bytes_read] = 0;
    }
    let null_pos = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    let str_byte = &buffer[..null_pos];
    let str_r = espc(str_byte);
    println!("char[] at 0x{:x}: \"{}\"", address, str_r);
}
