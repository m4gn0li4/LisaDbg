use std::{io, ptr};
use winapi::shared::minwindef::LPVOID;
use winapi::shared::ntdef::HANDLE;
use winapi::um::memoryapi::*;
use winapi::um::winnt::PAGE_EXECUTE_READWRITE;
use crate::dbg::{BASE_ADDR, SAVEINSN, SaveInsn};
use crate::OPTION;
use crate::utils::*;
use crate::pefile::function::FUNC_INFO;

pub unsafe fn restore_byte_of_brkpt(process_handle: HANDLE, breakpoint_addr: u64) {
    for insn in &*SAVEINSN {
        if insn.addr == breakpoint_addr {
            let mut old_protect = 0;
            let mut written = 0;
            if VirtualProtectEx(process_handle, insn.addr as LPVOID, 1, PAGE_EXECUTE_READWRITE, &mut old_protect) == 0 {
                eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> error when changing memory protection at address : {:#x}", breakpoint_addr);
                return;
            }
            if WriteProcessMemory(process_handle, insn.addr as LPVOID, &insn.last_oc as *const _ as LPVOID, 1, &mut written) == 0 {
                eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> error when writing to memory at address : {:#x} : {}", breakpoint_addr, io::Error::last_os_error());
            }else {
                println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Restored original byte at address: {:#x}", breakpoint_addr);
            }
            if VirtualProtectEx(process_handle, insn.addr as LPVOID, 1, old_protect, &mut old_protect) == 0 {
                eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> error while restoring memory protection at address: {:#x}", breakpoint_addr)
            }
            break;
        }
    }
}





pub unsafe fn set_breakpoint(process_handle: HANDLE, rva: u64) {
    let breakpoint_address = (BASE_ADDR + rva) as LPVOID;
    let mut old_protect = 0;
    let mut written = 0;
    let mut original_byte: u8 = 0;
    if VirtualProtectEx(process_handle, breakpoint_address, 1, PAGE_EXECUTE_READWRITE, &mut old_protect) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to change memory protection at address: 0x{:x} : {}", breakpoint_address as usize, io::Error::last_os_error());
        return;
    }
    if ReadProcessMemory(process_handle, breakpoint_address, &mut original_byte as *mut _ as LPVOID, 1, ptr::null_mut()) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to read memory at address: 0x{:x} : {}", breakpoint_address as usize, io::Error::last_os_error());
        return;
    }
    if original_byte == 0xcc { return }
    SAVEINSN.push(SaveInsn {
        addr: breakpoint_address as u64,
        last_oc: original_byte,
    });
    if WriteProcessMemory(process_handle, breakpoint_address, &0xcc as *const _ as LPVOID, 1, &mut written) == 0 || written != 1 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to write breakpoint at address: 0x{:x} : {}", breakpoint_address as usize, io::Error::last_os_error());
    }
    if VirtualProtectEx(process_handle, breakpoint_address, 1, old_protect, &mut old_protect) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to restore memory protection at address: 0x{:x} : {}", breakpoint_address as usize, io::Error::last_os_error());
        return;
    }
    println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Breakpoint set at address: {:#x} in memory", breakpoint_address as usize);
}






pub unsafe fn set_breakpoint_in_ret_func(process_handle: HANDLE, begin_addr: u64) {
    if let Some(func) = FUNC_INFO.iter().find(|s| s.BeginAddress == begin_addr as u32) {
        let va_begin_addr = func.BeginAddress as u64 + BASE_ADDR;
        let r_size = (func.EndAddress - func.BeginAddress) as usize;
        let mut byte_func = vec![0u8; r_size];
        let mut old_protect = 0;
        println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Setting breakpoints in function starting at address: 0x{:x}", va_begin_addr);
        if VirtualProtectEx(process_handle, va_begin_addr as LPVOID, r_size, PAGE_EXECUTE_READWRITE, &mut old_protect) == 0 {
            eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to change memory protection at address {:#x} for \"PAGE_READWRITE\"", va_begin_addr);
            return;
        }
        if ReadProcessMemory(process_handle, va_begin_addr as LPVOID, byte_func.as_mut_ptr() as LPVOID, r_size, &mut 0) == 0 {
            eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to read memory at address {:#x}", va_begin_addr);
            return;
        }
        if VirtualProtectEx(process_handle, va_begin_addr as LPVOID, r_size, old_protect, &mut 0) == 0 {
            eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to restore memory protection at address {:#x} : {}", va_begin_addr, io::Error::last_os_error());
            return;
        }
        for (i, b) in byte_func.iter().enumerate() {
            if *b == 0xc3 {
                let addr_rva = (i as u64 + va_begin_addr) - BASE_ADDR;
                OPTION.breakpoint_addr.push(addr_rva);
                set_breakpoint(process_handle, addr_rva);
            }
        }
    }
}



