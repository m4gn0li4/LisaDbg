use std::{io, mem, ptr};
use std::io::Write;
use regex::Regex;
use winapi::shared::minwindef::LPVOID;
use winapi::shared::ntdef::HANDLE;
use winapi::um::memoryapi::ReadProcessMemory;
use winapi::um::winnt::CONTEXT;
use crate::dbg::dbg_cmd::x64::info_reg::{ToValue, Value};
use crate::dbg::dbg_cmd::usages;
use crate::dbg::RealAddr;
use crate::pefile::NT_HEADER;
use crate::utils::*;
use crate::symbol::SYMBOLS_V;




trait Print {
    fn print_value(&self) -> String;
}


impl Print for u8 {
    fn print_value(&self) -> String {
        format!("{:#x}", self)
    }
}


impl Print for u16 {
    fn print_value(&self) -> String {
        format!("{:#x}", self)
    }
}


impl Print for u32 {
    fn print_value(&self) -> String {
        format!("{:#x}", self)
    }
}


impl Print for u64 {
    fn print_value(&self) -> String {
        format!("{:#x}", self)
    }
}


impl Print for usize {
    fn print_value(&self) -> String {
        format!("{:#x}", self)
    }
}


impl Print for f32 {
    fn print_value(&self) -> String {
        format!("{}", self)
    }
}


impl Print for f64 {
    fn print_value(&self) -> String {
        format!("{}", self)
    }
}


fn format_value<T: std::fmt::Debug + Default + Copy + PartialOrd + std::fmt::Display + std::fmt::LowerHex>(value: T) -> String {
    if value < T::default() {
        format!("{}", value)
    } else {
        format!("{:#x}", value)
    }
}


impl Print for i8 {
    fn print_value(&self) -> String {
        format_value(*self)
    }
}



impl Print for i16 {
    fn print_value(&self) -> String {
        format_value(*self)
    }
}


impl Print for i32 {
    fn print_value(&self) -> String {
        format_value(*self)
    }
}

impl Print for i64 {
    fn print_value(&self) -> String {
        format_value(*self)
    }
}




unsafe fn read_memory<T: Print + Default + Clone>(process_handle: HANDLE, address: usize, count_ptr: usize, size: usize, bytes_read: &mut usize){
    let r_size = size * mem::size_of::<T>();
    let mut result = vec![T::default(); size];
    let ptr_size = NT_HEADER.unwrap().get_size_of_arch();
    let mut addr_v = address;
    for i in 0..count_ptr {
        print!("{BYTES_COLOR}{:#x}{RESET_COLOR} -> ", addr_v);
        if i == count_ptr-1 {
            if ReadProcessMemory(process_handle, addr_v as LPVOID, result.as_mut_ptr() as LPVOID, r_size, bytes_read) == 0 {
                io::stdout().flush().unwrap();
                eprintln!("{ERR_COLOR}Bad ptr : {}{RESET_COLOR}", io::Error::last_os_error());
                return;
            }
            print!("{}{VALUE_COLOR}", if size > 1 {"\n[\n"}else {""});
            for (i, elm) in result.iter().enumerate() {
                println!("{}{}", elm.print_value(), if size > 1 || i != size - 1 {", "} else {""});
            }
            print!("{RESET_COLOR}{}", if size > 1 {"]\n"} else {""});
        }
        else {
            if ReadProcessMemory(process_handle, addr_v as LPVOID, ptr::addr_of_mut!(addr_v) as LPVOID, ptr_size, &mut 0) == 0 {
                io::stdout().flush().unwrap();
                eprintln!("{ERR_COLOR}bad ptr : {}{RESET_COLOR}", io::Error::last_os_error());
                return;
            }
        }
    }
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
    if !dtype.contains("str") && !dtype.contains("char[]"){
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

    let types_r = dtype.split('[').next().unwrap_or_default().split('*').next().unwrap_or_default();
    let count_ptr =  dtype.matches("*").count() + 1;

    unsafe {
        match types_r {
            "uint8_t" | "byte" | "u8" => read_memory::<u8>(process_handle, address, count_ptr, size, &mut bytes_read),
            "int8_t" | "i8" => read_memory::<i8>(process_handle, address, count_ptr, size, &mut bytes_read),
            "uint16_t" | "word" | "u16" => read_memory::<u16>(process_handle, address, count_ptr, size, &mut bytes_read),
            "int16_t" | "short" | "i16"  => read_memory::<i16>(process_handle, address, count_ptr, size, &mut bytes_read),
            "uint32_t" | "u32" | "dword" => read_memory::<u32>(process_handle, address, count_ptr, size, &mut bytes_read),
            "int32_t" | "int" | "long" | "i32" => read_memory::<i32>(process_handle, address,count_ptr, size, &mut bytes_read),
            "uint64_t" | "qword" | "u64" => read_memory::<u64>(process_handle, address, count_ptr, size, &mut bytes_read),
            "int64_t" | "i64" => read_memory::<i64>(process_handle, address,count_ptr, size, &mut bytes_read),
            "float" | "f32" => read_memory::<f32>(process_handle, address, count_ptr, size, &mut bytes_read),
            "double" | "f64" => read_memory::<f64>(process_handle, address, count_ptr, size, &mut bytes_read),
            "char" | "str" | "string" => read_string(process_handle, address, count_ptr, size, &mut bytes_read),
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




pub unsafe fn read_string(process_handle: HANDLE, address: usize, count_ptr: usize, size: usize, bytes_read: &mut usize) {
    let mut b_str = Vec::new();
    let mut addr_s = address;
    let ptr_size = NT_HEADER.unwrap().get_size_of_arch();
    let mut b = 1u8;
    for i in 0..count_ptr {
        print!("{BYTES_COLOR}{:#x}{RESET_COLOR} -> ", addr_s);
        if i == count_ptr-1 {
            let mut j = 0;
            loop {
                if ReadProcessMemory(process_handle, (addr_s + j) as LPVOID, ptr::addr_of_mut!(b) as LPVOID, 1, &mut 0) == 0 {
                    io::stdout().flush().unwrap();
                    eprintln!("{ERR_COLOR}Bad ptr : {}", io::Error::last_os_error());
                    return;
                }
                if size != 0 && j == size || size == 0 && b == 0 {
                    break
                }
                b_str.push(b);
                *bytes_read += 1;
                j+=1;
            }
            let str_byte = &b_str[..if size != 0 {size} else {b_str.len()}];
            let str_r = espc(str_byte);
            println!("\"{}\"", str_r);
            return;
        }else {
            if ReadProcessMemory(process_handle, addr_s as LPVOID, ptr::addr_of_mut!(addr_s) as LPVOID, ptr_size, &mut 0) == 0 {
                io::stdout().flush().unwrap();
                eprintln!("{ERR_COLOR}bad ptr : {}{RESET_COLOR}", io::Error::last_os_error());
                return;
            }
        }
    }
}
