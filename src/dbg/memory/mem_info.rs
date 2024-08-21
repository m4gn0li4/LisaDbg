use std::{io, mem};
use winapi::shared::minwindef::LPVOID;
use winapi::um::memoryapi::VirtualQueryEx;
use winapi::um::winnt::*;
use crate::utils::{ERR_COLOR, get_addr_va};

pub fn handle_mem_info(linev: &[&str], process_handle: HANDLE, ctx: CONTEXT) {
    if linev.len() != 2 {
        println!("caca");
        return;
    }
    let target = linev[1];
    let addr = match get_addr_va(target, ctx) {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };
    unsafe {
        let mut mem_info: MEMORY_BASIC_INFORMATION = mem::zeroed();
        let query_result = VirtualQueryEx(process_handle, addr as LPVOID, &mut mem_info, mem::size_of::<MEMORY_BASIC_INFORMATION>());
        if query_result == 0 {
            eprintln!("{ERR_COLOR}failed to get memory information for address {:#x} : {}", addr, io::Error::last_os_error());
            return;
        }
        print_mem_info(mem_info);
    }
}




pub fn print_mem_info(mem_info: MEMORY_BASIC_INFORMATION) {
    println!("  Memory information :");
    println!("       Base Address    : {:#x}", mem_info.BaseAddress as usize);
    println!("       Allocation Base : {:#x}", mem_info.AllocationBase as usize);
    println!("       Region Size     : {:#x}", mem_info.RegionSize);
    println!("       State           : {}",
             if mem_info.State == MEM_COMMIT { "Committed" }
             else if mem_info.State == MEM_RESERVE { "Reserved" }
             else if mem_info.State == MEM_FREE {"free"}
             else {"unknow"});
    println!("       Protect         : {}", match mem_info.Protect {
        PAGE_READONLY => "Read Only",
        PAGE_READWRITE => "Read/Write",
        PAGE_EXECUTE => "Execute",
        PAGE_EXECUTE_READ => "Execute/Read",
        PAGE_EXECUTE_READWRITE => "Execute/Read/Write",
        _ => "Unknown",
    });
    println!("       Type            : {}", match mem_info.Type {
        MEM_IMAGE => "Image",
        MEM_MAPPED => "Mapped",
        MEM_PRIVATE => "Private",
        _ => "Unknown",
    });
}