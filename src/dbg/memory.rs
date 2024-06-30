use std::{io, ptr};
use winapi::um::memoryapi::{ReadProcessMemory, VirtualAllocEx, VirtualProtectEx, WriteProcessMemory};
use winapi::um::winnt::{HANDLE, MEM_COMMIT, MEM_RESERVE, PAGE_EXECUTE_READWRITE};
use winapi::shared::minwindef::{LPCVOID, LPVOID};
use crate::dbg::{BASE_ADDR, INSNBRPT, InsnInBrpt};
use crate::log::*;
use crate::{OPTION, pefile};
use crate::pefile::function::CrtFunc;

pub unsafe fn restore_byte_of_brkpt(process_handle: HANDLE, breakpoint_addr: u64) {
    for insn in &*INSNBRPT {
        if insn.addr == breakpoint_addr {
            let mut old_protect = 0;
            let mut written = 0;
            if VirtualProtectEx(process_handle, insn.addr as LPVOID, 1, PAGE_EXECUTE_READWRITE, &mut old_protect) == 0 {
                eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> error when changing memory protection at adress : {:#x}", breakpoint_addr);
                return;
            }
            if WriteProcessMemory(process_handle, insn.addr as LPVOID, &insn.last_oc as *const _ as LPCVOID, 1, &mut written) == 0 {
                eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> error when writing to memory at adress : {:#x}", breakpoint_addr);
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


pub unsafe fn set_breakpoint(process_handle: HANDLE, base_address: LPVOID, rva: u64) {
    let breakpoint_address = (base_address as u64 + rva) as LPVOID;
    let mut old_protect = 0;
    let mut written = 0;
    let mut original_byte: u8 = 0;
    if ReadProcessMemory(process_handle, breakpoint_address, &mut original_byte as *mut _ as LPVOID, 1, ptr::null_mut()) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to read memory at address: 0x{:x}", breakpoint_address as usize);
        return;
    }
    if original_byte == 0xcc { return }
    INSNBRPT.push(InsnInBrpt {
        addr: breakpoint_address as u64,
        last_oc: original_byte,
    });
    if VirtualProtectEx(process_handle, breakpoint_address, 1, PAGE_EXECUTE_READWRITE, &mut old_protect) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to change memory protection at address: 0x{:x}", breakpoint_address as usize);
        return;
    }
    if WriteProcessMemory(process_handle, breakpoint_address, &0xCC as *const _ as LPCVOID, 1, &mut written) == 0 || written != 1 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to write breakpoint at address: 0x{:x}", breakpoint_address as usize);
    }
    if VirtualProtectEx(process_handle, breakpoint_address, 1, old_protect, &mut old_protect) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to restore memory protection at address: 0x{:x}", breakpoint_address as usize);
        return;
    }
    println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Breakpoint set at address: {:#x} in memory", breakpoint_address as usize);
}




pub unsafe fn set_breakpoint_in_ret_func(process_handle: HANDLE, begin_addr: u64) {
    if let Some(func) = pefile::function::FUNC_INFO.iter().find(|func|func.BeginAddress == begin_addr as u32) {
        let end_func = func.EndAddress as u64 + BASE_ADDR;
        let mut i_addr = func.BeginAddress as u64 + BASE_ADDR;
        while i_addr <= end_func {
            let mut old_protect = 0;
            if VirtualProtectEx(process_handle, i_addr as LPVOID, 1, PAGE_EXECUTE_READWRITE, &mut old_protect) == 0 {
                eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> error when removing memory protection at address : {:#x}", i_addr);
                continue
            }
            let mut byte: u8 = 0;
            if ReadProcessMemory(process_handle, i_addr as LPVOID, &mut byte as *mut _ as LPVOID, 1, ptr::null_mut()) == 0 {
                eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> error when reading byte at address : {:#x}", i_addr);
                continue
            }
            if byte == 0xc3 {
                OPTION.breakpoint_addr.push(i_addr - BASE_ADDR);
                set_breakpoint(process_handle, BASE_ADDR as LPVOID, i_addr - BASE_ADDR);
            }
            if VirtualProtectEx(process_handle, i_addr as LPVOID, 1, old_protect, &mut old_protect) == 0 {
                eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to restore memory protection at address : {:#x}", i_addr);
                return;
            }
            i_addr += 1;
        }
    }
}


pub unsafe fn set_addr_over(process_handle: HANDLE, over_func: u64) {
    let mut old_protect = 0;
    let over_func = over_func + BASE_ADDR;
    if VirtualProtectEx(process_handle, over_func as LPVOID, 1, PAGE_EXECUTE_READWRITE, &mut old_protect) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> an error occurred while removing memory protection at address {:#x}", over_func);
    }
    if WriteProcessMemory(process_handle, over_func as LPVOID, &0xc3 as *const _ as LPVOID, 1, &mut 0) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> an error occurred while writing to memory at address {:#x}", over_func);
    }else {
        println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> writing memory at address {:#x} is successfull", over_func);
    }
    if VirtualProtectEx(process_handle, over_func as LPVOID, 1, old_protect, &mut old_protect) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> an error occurred while restauring memory protection at address {:#x}", over_func);
        return;
    }
}


pub unsafe fn set_cr_function(process_handle: HANDLE, crt_func: &mut CrtFunc) {
    let addr_func = VirtualAllocEx(process_handle, 0 as LPVOID, 11, MEM_COMMIT | MEM_RESERVE, PAGE_EXECUTE_READWRITE);
    crt_func.address = addr_func as u64;
    if addr_func.is_null() {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> error while allocating memory space for function {} : {}", crt_func.name, io::Error::last_os_error());
        return
    }
    let mut code = vec![0x48, 0xb8];
    code.extend_from_slice(&crt_func.ret_value.to_le_bytes());
    code.push(0xc3);
    let mut written = 0;
    if WriteProcessMemory(process_handle, addr_func, code.as_mut_ptr() as LPVOID, 11, &mut written) == 0 || written != 11 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> error when writing bytes of function {} at address {:#x} : {}", crt_func.name, crt_func.address, io::Error::last_os_error());
        return;
    }
    println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> the function {} was created successfully at address {:#x}", crt_func.name, crt_func.address);
}