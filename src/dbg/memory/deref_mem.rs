use crate::command::def::structs::{StructP, TypeP};
use crate::dbg::dbg_cmd::usages;
use crate::dbg::dbg_cmd::x64::info_reg::{ToValue, Value};
use crate::dbg::RealAddr;
use crate::pefile::NT_HEADER;
use crate::symbol::SYMBOLS_V;
use crate::utils::*;
use crate::ALL_ELM;
use regex::Regex;
use std::io::Write;
use std::{io, ptr};
use winapi::shared::minwindef::LPVOID;
use winapi::shared::ntdef::HANDLE;
use winapi::um::memoryapi::ReadProcessMemory;
use winapi::um::winnt::CONTEXT;

unsafe fn read_memory<T: Print + Default + Clone>(h_proc: HANDLE, address: usize, count_ptr: usize, array_cout: usize, field_name: &str, bytes_read: &mut usize) {
    let r_size = array_cout * size_of::<T>();
    let mut result = vec![T::default(); array_cout];
    let ptr_size = NT_HEADER.unwrap().get_size_of_arch();
    let mut addr_v = address;

    for i in 0..count_ptr {
        print!("{BYTES_COLOR}{:#x}{RESET_COLOR} -> ", addr_v);

        if i == count_ptr - 1 {
            if ReadProcessMemory(h_proc, addr_v as LPVOID, result.as_mut_ptr() as LPVOID, r_size, bytes_read) == 0 {
                io::stdout().flush().unwrap();
                eprintln!("{ERR_COLOR}Bad ptr : {}{RESET_COLOR}", io::Error::last_os_error());
                return;
            }

            print!("{field_name}: {}{VALUE_COLOR}", if array_cout > 1 { "\n[\n" } else { "" });

            for (i, elm) in result.iter().enumerate() {
                print!("{}", elm.print_value());

                if i != array_cout - 1 {
                    print!("{RESET_COLOR},{VALUE_COLOR}   ");
                }

                if (i + 1) % 4 == 0 && i != array_cout - 1 {
                    print!("\n");
                }
            }
            println!("{RESET_COLOR}{}", if array_cout > 1 { "\n]\n" } else { "" });
            io::stdout().flush().unwrap();
        } else {
            if ReadProcessMemory(h_proc, addr_v as LPVOID, ptr::addr_of_mut!(addr_v) as LPVOID, ptr_size, &mut 0) == 0 {
                io::stdout().flush().unwrap();
                eprintln!("{ERR_COLOR}bad ptr : {}{RESET_COLOR}", io::Error::last_os_error());
                return;
            }
        }
    }
}

unsafe fn match_fi(type_p: TypeP, h_proc: HANDLE, addr_n: usize, cout_ptr: usize, field_name: &str) {
    match type_p {
        TypeP::U8(cout) => read_memory::<u8>(h_proc, addr_n, cout_ptr, cout, field_name, &mut 0),
        TypeP::I8(cout) => read_memory::<i8>(h_proc, addr_n, cout_ptr, cout, field_name, &mut 0),
        TypeP::U16(cout) => read_memory::<u16>(h_proc, addr_n, cout_ptr, cout, field_name, &mut 0),
        TypeP::I16(cout) => read_memory::<i16>(h_proc, addr_n, cout_ptr, cout, field_name, &mut 0),
        TypeP::U32(cout) => read_memory::<u32>(h_proc, addr_n, cout_ptr, cout, field_name, &mut 0),
        TypeP::I32(cout) => read_memory::<i32>(h_proc, addr_n, cout_ptr, cout, field_name, &mut 0),
        TypeP::U64(cout) => read_memory::<u64>(h_proc, addr_n, cout_ptr, cout, field_name, &mut 0),
        TypeP::I64(cout) => read_memory::<i64>(h_proc, addr_n, cout_ptr, cout, field_name, &mut 0),
        TypeP::F32(cout) => read_memory::<f32>(h_proc, addr_n, cout_ptr, cout, field_name, &mut 0),
        TypeP::F64(cout) => read_memory::<f64>(h_proc, addr_n, cout_ptr, cout, field_name, &mut 0),
        TypeP::Ptr(ptrs, cout) => {
            for _ in 0..cout {
                match_fi(*ptrs.type_deref.clone(), h_proc, addr_n, ptrs.cout_ptr + 1, field_name)
            }
        }
        TypeP::Bool(cout) => read_memory::<bool>(h_proc, addr_n, cout_ptr, cout, field_name, &mut 0),
        TypeP::Char(cout) => read_string(h_proc, addr_n, cout_ptr, cout, &mut 0, false),
        TypeP::Structs(field_s, name) => read_memory_for_struct(h_proc, addr_n, 1, field_s, &name),
        _ => {}
    }
}

unsafe fn read_memory_for_struct(h_proc: HANDLE, address: usize, cout_ptr: usize, struct_def: Vec<StructP>, name_struct: &str) {
    let mut addr_n = address;
    let size_ptr = NT_HEADER.unwrap().get_size_of_arch();
    for i in 0..cout_ptr {
        if i == cout_ptr - 1 {
            println!("struct {name_struct} {{");
            for field in &struct_def {
                match_fi(field.type_p.clone(), h_proc, addr_n, 1, &field.name_field);
                addr_n += field.type_p.get_size();
            }
            println!("}}");
        } else {
            if ReadProcessMemory(h_proc, addr_n as LPVOID, ptr::addr_of_mut!(addr_n) as LPVOID, size_ptr, &mut 0) == 0 {
                eprintln!("{ERR_COLOR}Bad ptr : {}{RESET_COLOR}", io::Error::last_os_error());
                return;
            }
        }
    }
}

pub fn handle_deref(linev: &[&str], ctx: CONTEXT, h_proc: HANDLE) {
    if linev.len() < 3 {
        eprintln!("{}", usages::USAGE_DEREF);
        return;
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
        } else {
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
    if let Err(err) = deref_memory(h_proc, dtype, address as usize) {
        eprintln!("{ERR_COLOR}{}{RESET_COLOR}", err)
    }
}

pub fn deref_memory(h_proc: HANDLE, dtype: &str, address: usize) -> Result<(), String> {
    let mut bytes_read = 0;
    let re = Regex::new(r"\[(.*?)]").unwrap();
    let mut size = 1;
    let str_s = ["str", "wstr", "char[]", "wchar[]", "str[]", "wstr[]"];
    if !str_s.contains(&dtype.replace("*", "").as_str()) {
        for cap in re.captures_iter(dtype) {
            if let Some(numd) = cap.get(1) {
                match str_to::<usize>(numd.as_str()) {
                    Ok(num) => size = num,
                    Err(e) => return Err(e.to_string()),
                }
            }
        }
    } else {
        size = 0;
    }

    let types_r = dtype.split('[').next().unwrap_or_default().split('*').next().unwrap_or_default();
    let count_ptr = dtype.matches("*").count() + 1;
    unsafe {
        match types_r {
            "bool" => read_memory::<bool>(h_proc, address, size, bytes_read, "", &mut bytes_read),
            "uint8_t" | "byte" | "u8" => read_memory::<u8>(h_proc, address, count_ptr, size, "", &mut bytes_read),
            "int8_t" | "i8" => read_memory::<i8>(h_proc, address, count_ptr, size, "", &mut bytes_read),
            "uint16_t" | "word" | "u16" => read_memory::<u16>(h_proc, address, count_ptr, size, "", &mut bytes_read),
            "int16_t" | "short" | "i16" => read_memory::<i16>(h_proc, address, count_ptr, size, "", &mut bytes_read),
            "uint32_t" | "u32" | "dword" => read_memory::<u32>(h_proc, address, count_ptr, size, "", &mut bytes_read),
            "int32_t" | "int" | "long" | "i32" => read_memory::<i32>(h_proc, address, count_ptr, size, "", &mut bytes_read),
            "uint64_t" | "qword" | "u64" => read_memory::<u64>(h_proc, address, count_ptr, size, "", &mut bytes_read),
            "int64_t" | "i64" => read_memory::<i64>(h_proc, address, count_ptr, size, "", &mut bytes_read),
            "float" | "f32" => read_memory::<f32>(h_proc, address, count_ptr, size, "", &mut bytes_read),
            "double" | "f64" => read_memory::<f64>(h_proc, address, count_ptr, size, "", &mut bytes_read),
            "size_t" => read_memory::<usize>(h_proc, address, count_ptr, size, "", &mut bytes_read),
            "char" | "str" | "string" => read_string(h_proc, address, count_ptr, size, &mut bytes_read, false),
            "wchar" | "wstr" | "wstring" => read_string(h_proc, address, count_ptr, size, &mut bytes_read, true),
            _ => {
                if ALL_ELM.struct_def.iter().any(|s| s.get_name_of_struct() == types_r) {
                    for structs in &ALL_ELM.struct_def {
                        read_memory_for_struct(h_proc, address, count_ptr, structs.get_field_of_struct(), &structs.get_name_of_struct())
                    }
                } else {
                    return Err(format!("Unknown type: {}", dtype));
                }
            }
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



pub unsafe fn read_string(h_proc: HANDLE, address: usize, count_ptr: usize, size: usize, bytes_read: &mut usize, wstr: bool) {
    let mut b_str = Vec::new();
    let mut addr_s = address;
    let ptr_size = NT_HEADER.unwrap().get_size_of_arch();
    let mut b = 0u8;
    let add_value = if wstr { 2 } else { 1 };
    for i in 0..count_ptr {
        print!("{BYTES_COLOR}{:#x}{RESET_COLOR} -> ", addr_s);
        if i == count_ptr - 1 {
            let mut j = 0;
            loop {
                if ReadProcessMemory(h_proc, (addr_s + j * add_value) as LPVOID, ptr::addr_of_mut!(b) as LPVOID, 1, &mut 0) == 0 {
                    io::stdout().flush().unwrap();
                    eprintln!("{ERR_COLOR}Bad ptr : {}", io::Error::last_os_error());
                    return;
                }
                if size != 0 && j == size || size == 0 && b == 0 {
                    break;
                }
                b_str.push(b);
                *bytes_read += 1;
                j += 1;
            }
            let str_byte = &b_str[..if size != 0 { size } else { b_str.len() }];
            let str_r = espc(str_byte);
            println!("{}\"{}\"", if wstr { "L" } else { "" }, str_r);
            return;
        } else {
            if ReadProcessMemory(h_proc, addr_s as LPVOID, ptr::addr_of_mut!(addr_s) as LPVOID, ptr_size, &mut 0) == 0 {
                io::stdout().flush().unwrap();
                eprintln!("{ERR_COLOR}bad ptr : {}{RESET_COLOR}", io::Error::last_os_error());
                return;
            }
        }
    }
}
