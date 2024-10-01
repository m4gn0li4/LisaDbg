use crate::dbg::dbg_cmd::usages::USAGE_FIND;
use crate::utils::*;
use std::{io, mem};
use winapi::shared::minwindef::LPVOID;
use winapi::shared::ntdef::HANDLE;
use winapi::um::memoryapi::{ReadProcessMemory, VirtualQueryEx};
use winapi::um::winnt::MEMORY_BASIC_INFORMATION;

pub fn handle_find(linev: &[&str], h_proc: HANDLE) {
    if linev.len() < 5 {
        println!("{USAGE_FIND}");
        return;
    }
    let (begin_addr, end_addr) = match (str_to::<u64>(linev[2]), str_to::<u64>(linev[3])) {
        (Ok(addr1), Ok(addr2)) => (addr1, addr2),
        (Err(e1), Ok(_)) => {
            eprintln!("{ERR_COLOR}failed to parse begin addr : {e1}{RESET_COLOR}");
            return;
        }
        (Ok(_), Err(e2)) => {
            eprintln!("{ERR_COLOR}failed to parse end addr : {e2}{RESET_COLOR}");
            return;
        }
        (Err(e1), Err(e2)) => {
            eprintln!("{ERR_COLOR}failed to parse begin & end addr, 1 : {e1}, 2 : {e2}{RESET_COLOR}");
            return;
        }
    };

    match linev[1] {
        "uint8_t" | "u8" | "byte" => find_seq_byte::<u8>(h_proc, begin_addr, end_addr, linev[4..].join(" ")),
        "i8" | "int8_t" => find_seq_byte::<i8>(h_proc, begin_addr, end_addr, linev[4..].join(" ")),
        "char" => find_seq_byte::<AsChar>(h_proc, begin_addr, end_addr, linev[4..].join(" ")),
        "uint16_t" | "u16" | "word" => find_seq_byte::<u16>(h_proc, begin_addr, end_addr, linev[4..].join(" ")),
        "int16_t" | "i16" | "short" => find_seq_byte::<i16>(h_proc, begin_addr, end_addr, linev[4..].join(" ")),
        "uint32_t" | "u32" | "dword" => find_seq_byte::<u32>(h_proc, begin_addr, end_addr, linev[4..].join(" ")),
        "int32_t" | "i32" | "int" | "long" => find_seq_byte::<i32>(h_proc, begin_addr, end_addr, linev[4..].join(" ")),
        "uint64_t" | "u64" | "qword" => find_seq_byte::<u64>(h_proc, begin_addr, end_addr, linev[4..].join(" ")),
        "int64_t" | "i64" | "long long" => find_seq_byte::<i64>(h_proc, begin_addr, end_addr, linev[4..].join(" ")),
        _ => eprintln!("{ERR_COLOR}unknow type : {}{RESET_COLOR}", linev[1]),
    }
}

fn find_seq_byte<T: ToType + Default + Clone + std::fmt::Debug + PartialEq + std::fmt::Display>(h_proc: HANDLE, beg_addr: u64, end_addr: u64, args: String) {
    let wordv = args.split(",").map(|s| s.trim()).collect::<Vec<&str>>();
    let mut result = Vec::new();

    for word in wordv {
        if word.starts_with("\"") || word.starts_with("'") {
            let new_word = word[1..word.len() - 1].trim();
            for c in new_word.chars() {
                result.push(T::from_char(c));
            }
        } else {
            match T::from_str_value(word) {
                Ok(val) => result.push(val),
                Err(e) => eprintln!("{ERR_COLOR}failed to parse value : {e}{RESET_COLOR}"),
            }
        }
    }

    let size_mem = if end_addr != 0 {
        if beg_addr < end_addr {
            (end_addr - beg_addr) as usize
        } else {
            eprintln!("{ERR_COLOR}you cannot specify a start address greater than the end address{RESET_COLOR}");
            return;
        }
    } else {
        unsafe {
            let mut mem_info = mem::zeroed();
            if VirtualQueryEx(h_proc, beg_addr as LPVOID, &mut mem_info, size_of::<MEMORY_BASIC_INFORMATION>()) == 0 {
                eprintln!("{ERR_COLOR}failed to query memory info for addr {:#x}: {}", beg_addr, io::Error::last_os_error());
                return;
            }
            mem_info.RegionSize
        }
    };

    if size_of::<T>() > size_mem {
        eprintln!("{ERR_COLOR}the size of the memory to analyze is smaller than a single {}{RESET_COLOR}", std::any::type_name::<T>());
        return;
    }

    let mut plage_mem = vec![T::default(); size_mem / size_of::<T>()];
    unsafe {
        let mut bytes_read = 0;
        if ReadProcessMemory(h_proc, beg_addr as LPVOID, plage_mem.as_mut_ptr() as LPVOID, size_mem, &mut bytes_read) == 0 {
            eprintln!("{ERR_COLOR}failed to read memory {size_mem} bytes at address {:#x}: {}", beg_addr, io::Error::last_os_error());
            return;
        }

        if bytes_read == 0 {
            eprintln!("{ERR_COLOR}No bytes read from memory{RESET_COLOR}");
            return;
        }
    }

    if result.len() > plage_mem.len() {
        eprintln!("{ERR_COLOR}you specified too many elements for the elements{RESET_COLOR}");
        return;
    }

    let mut found_addr = Vec::new();
    let addr_find = beg_addr;
    for i in 0..plage_mem.len() {
        if plage_mem[i] == result[0] && plage_mem[i..].len() >= result.len() {
            if plage_mem[i..i + result.len()] == *result {
                found_addr.push(addr_find + (i * size_of::<T>()) as u64);
            }
        }
    }

    if found_addr.is_empty() {
        eprintln!("{ERR_COLOR}element {:?} not found{RESET_COLOR}", result);
    } else {
        for addr in found_addr {
            println!("{DBG_COLOR}element found at address {VALUE_COLOR}{:#x}{RESET_COLOR}", addr);
        }
    }
}
