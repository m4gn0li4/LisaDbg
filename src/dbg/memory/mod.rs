use crate::dbg::{SaveInsn, BASE_ADDR, SAVEINSN};
use crate::symbol;
use crate::utils::*;
use std::io;
use winapi::shared::minwindef::LPVOID;
use winapi::shared::ntdef::HANDLE;
use winapi::um::memoryapi::{ReadProcessMemory, VirtualProtectEx, WriteProcessMemory};
use winapi::um::winnt::PAGE_EXECUTE_READWRITE;

pub mod breakpoint;
pub mod deref_mem;
pub mod finder;
pub mod func;
pub mod mem_info;
pub mod set;
pub mod stack;
pub mod watchpoint;

pub unsafe fn set_addr_over(h_proc: HANDLE, over_func: u64) {
    let mut old_protect = 0;
    let mut old_byte = 0u8;
    let over_func = over_func + BASE_ADDR;
    if VirtualProtectEx(h_proc, over_func as LPVOID, 1, PAGE_EXECUTE_READWRITE, &mut old_protect) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> an error occurred while removing memory protection at address {:#x}", over_func);
    }
    if ReadProcessMemory(h_proc, over_func as LPVOID, &mut old_byte as *mut _ as LPVOID, 1, &mut 0) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to read memory at address {:#x} : {}", over_func, io::Error::last_os_error());
        return;
    }
    if WriteProcessMemory(h_proc, over_func as LPVOID, &0xc3u8 as *const u8 as LPVOID, 1, &mut 0) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> an error occurred while writing to memory at address {:#x}", over_func);
    } else {
        println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> now, function {} will no longer run", func_format(over_func));
        SAVEINSN.push(SaveInsn {
            last_oc: old_byte,
            addr: over_func,
        });
    }
    if VirtualProtectEx(h_proc, over_func as LPVOID, 1, old_protect, &mut old_protect) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> an error occurred while restauring memory protection at address {:#x}", over_func);
        return;
    }
}

unsafe fn func_format(addr: u64) -> String {
    if let Some(sym) = symbol::SYMBOLS_V.symbol_file.iter().find(|s| s.offset + BASE_ADDR as i64 == addr as i64) {
        format!("{} at address {:#x}", sym.name, addr)
    } else {
        format!("at address {:#x}", addr)
    }
}
