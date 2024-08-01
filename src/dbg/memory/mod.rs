use std::io;
use winapi::shared::minwindef::LPVOID;
use winapi::shared::ntdef::HANDLE;
use winapi::um::memoryapi::{ReadProcessMemory, VirtualAllocEx, VirtualProtectEx, WriteProcessMemory};
use winapi::um::winnt::{MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE};
use crate::dbg::{BASE_ADDR, SAVEINSN, SaveInsn};
use crate::pefile::function::CrtFunc;
use crate::pefile::NT_HEADER;
use crate::symbol;
use crate::log::*;

pub mod breakpoint;
pub mod watchpoint;
pub mod stack;

pub unsafe fn set_addr_over(process_handle: HANDLE, over_func: u64) {
    let mut old_protect = 0;
    let mut old_byte = 0u8;
    let over_func = over_func + BASE_ADDR;
    if VirtualProtectEx(process_handle, over_func as LPVOID, 1, PAGE_EXECUTE_READWRITE, &mut old_protect) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> an error occurred while removing memory protection at address {:#x}", over_func);
    }
    if ReadProcessMemory(process_handle, over_func as LPVOID, &mut old_byte as *mut _ as LPVOID, 1, &mut 0) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to read memory at address {:#x} : {}", over_func, io::Error::last_os_error());
        return;
    }
    if WriteProcessMemory(process_handle, over_func as LPVOID, &0xc3 as *const _ as LPVOID, 1, &mut 0) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> an error occurred while writing to memory at address {:#x}", over_func);
    }else {
        println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> now, function {} will no longer run", func_format(over_func));
        SAVEINSN.push(SaveInsn {
            last_oc: old_byte,
            addr: over_func
        });
    }
    if VirtualProtectEx(process_handle, over_func as LPVOID, 1, old_protect, &mut old_protect) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> an error occurred while restauring memory protection at address {:#x}", over_func);
        return;
    }
}



unsafe fn func_format(addr: u64) -> String {
    if let Some(sym) = symbol::SYMBOLS_V.symbol_file.iter().find(|s|s.offset + BASE_ADDR as i64 == addr as i64) {
        format!("{} at address {:#x}", sym.name, addr)
    }else {
        format!("at address {:#x}", addr)
    }
}



pub unsafe fn set_cr_function(process_handle: HANDLE, crt_func: &mut CrtFunc) {
    let mut code = Vec::new();
    let bitness = NT_HEADER.unwrap().get_bitness();
    if bitness == 32 {
        code.push(0xb8);
        code.push(0x0a);
    } else {
        code.push(0x48);
        code.push(0xb8);
    }
    let le_byte_value = if bitness == 32 {
        (crt_func.ret_value as u32).to_le_bytes().to_vec()
    } else {
        crt_func.ret_value.to_le_bytes().to_vec()
    };
    code.extend_from_slice(&le_byte_value);
    code.push(0xc3);

    let addr_func = VirtualAllocEx(process_handle, 0 as LPVOID, code.len(), MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
    if addr_func.is_null() {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> error while allocating memory space for function {} : {}", crt_func.name, io::Error::last_os_error());
        return
    }
    crt_func.address = addr_func as u64;
    if let Some(sym) = symbol::SYMBOLS_V.symbol_file.iter_mut().find(|sym|sym.name == crt_func.name) {
        sym.offset = addr_func as i64;
    }
    let mut written = 0;
    if WriteProcessMemory(process_handle, addr_func, code.as_ptr() as LPVOID, code.len(), &mut written) == 0 || written != 11 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> error when writing bytes of function {} at address {:#x} : {}", crt_func.name, crt_func.address, io::Error::last_os_error());
        return;
    }
    println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> the function {} was created successfully at address {:#x}", crt_func.name, crt_func.address);
}