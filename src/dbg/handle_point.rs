use crate::cli::{AfterB, ALL_ELM};
use crate::command::hook::Hook;
use crate::dbg::{memory, RealAddr, SaveInsn, BASE_ADDR, SAVEINSN};
use crate::pefile::{NtHeaders, NT_HEADER};
use crate::utils::*;
use std::{io, mem, ptr};
use winapi::shared::minwindef::{FALSE, LPVOID};
use winapi::um::memoryapi::{ReadProcessMemory, WriteProcessMemory};
use winapi::um::minwinbase::DEBUG_EVENT;
use winapi::um::processthreadsapi::{GetThreadContext, OpenThread, SetThreadContext, TerminateProcess};
use winapi::um::winbase::{Wow64GetThreadContext, Wow64SetThreadContext};
use winapi::um::winnt::{CONTEXT, CONTEXT_ALL, HANDLE, THREAD_ALL_ACCESS, THREAD_GET_CONTEXT, THREAD_SET_CONTEXT, WOW64_CONTEXT, WOW64_CONTEXT_ALL};



pub fn handle_single_step(debug_event: DEBUG_EVENT, b_addr: u64, h_proc: HANDLE, c_dbg: &mut bool) {
    unsafe {
        let mut h_thread = OpenThread(THREAD_ALL_ACCESS, 0, debug_event.dwThreadId);
        if h_thread.is_null() {
            eprintln!("[{ERR_COLOR}Error{ERR_COLOR}] -> failed to open thread {} : {}", debug_event.dwThreadId, io::Error::last_os_error());
            return;
        }
        match NT_HEADER.unwrap() {
            NtHeaders::Headers32(_) => {
                let mut ctx: WOW64_CONTEXT = mem::zeroed();
                ctx.ContextFlags = WOW64_CONTEXT_ALL;
                if Wow64GetThreadContext(h_thread, &mut ctx) == 0 {
                    eprintln!("[{ERR_COLOR}Error{ERR_COLOR}] -> failed to get thread context of thread {} : {}", debug_event.dwThreadId, io::Error::last_os_error());
                    return;
                }
                if ALL_ELM.watchpts.iter().any(|w|w.real_addr32(ctx) == b_addr as u32) {
                    memory::watchpoint::handle_watchpoint32(debug_event, h_proc, &mut h_thread, &mut ctx, c_dbg);
                }
            }
            NtHeaders::Headers64(_) => {
                let mut ctx: CONTEXT = mem::zeroed();
                ctx.ContextFlags = CONTEXT_ALL;
                if GetThreadContext(h_thread, &mut ctx) == 0 {
                    eprintln!("[{ERR_COLOR}Error{ERR_COLOR}] -> failed to get thread context of thread {} : {}", debug_event.dwThreadId, io::Error::last_os_error());
                    return;
                }
                if ALL_ELM.watchpts.iter().any(|w|w.real_addr64(ctx) == b_addr) {
                    memory::watchpoint::handle_watchpoint64(debug_event, h_proc, &mut h_thread, &mut ctx, c_dbg);
                }
            }
        }
    }
}



pub unsafe fn handle_after_b(h_proc: HANDLE, ab: AfterB, c_dbg: &mut bool, debug_event: DEBUG_EVENT) {
    if WriteProcessMemory(h_proc, ab.after_b as LPVOID, ptr::addr_of!(ab.last_oc) as LPVOID, 1, &mut 0) == 0 {
        eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> failed to restore the original byte at address {:#x} : {}", ab.after_b, io::Error::last_os_error());
        *c_dbg = false;
        return;
    }
    let h_thread = OpenThread(THREAD_ALL_ACCESS, 0, debug_event.dwThreadId);
    if h_thread.is_null() {
        eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> Failed to open thread {}: {}{RESET_COLOR}", debug_event.dwThreadId, io::Error::last_os_error());
        *c_dbg = false;
        return;
    }
    match NT_HEADER.unwrap() {
        NtHeaders::Headers32(_) => {
            let mut ctx: WOW64_CONTEXT = mem::zeroed();
            ctx.ContextFlags = WOW64_CONTEXT_ALL;
            if Wow64GetThreadContext(h_thread, &mut ctx) == 0 {
                eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> Failed to get thread context: {}", io::Error::last_os_error());
                *c_dbg = false;
                return;
            }
            ctx.Eip -= 1;
            if Wow64SetThreadContext(h_thread, &ctx) == 0 {
                eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> Failed to set thread context: {}", io::Error::last_os_error());
                *c_dbg = false;
                return;
            }
        }
        NtHeaders::Headers64(_) => {
            let mut ctx: CONTEXT = mem::zeroed();
            ctx.ContextFlags = CONTEXT_ALL;
            if GetThreadContext(h_thread, &mut ctx) == 0 {
                eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> Failed to get thread context: {}", io::Error::last_os_error());
                *c_dbg = false;
                return;
            }
            ctx.Rip -= 1;
            if SetThreadContext(h_thread, &ctx) == 0 {
                eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> Failed to set thread context: {}", io::Error::last_os_error());
                *c_dbg = false;
                return;
            }
        }
    }
    let mut last_oc = 0u8;
    if ReadProcessMemory(h_proc, ab.last_addr_b as LPVOID, ptr::addr_of_mut!(last_oc) as LPVOID, 1, &mut 0) == 0 {
        eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}) -> failed to read memory at address {:#x} : {}", ab.last_addr_b, io::Error::last_os_error());
        *c_dbg = false;
        return;
    }
    SAVEINSN.push(SaveInsn {last_oc, addr: ab.last_addr_b});
    if WriteProcessMemory(h_proc, ab.last_addr_b as LPVOID, &0xccu8 as *const u8 as LPVOID, 1, &mut 0) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to write breakpoint at address {:#x} : {}", ab.last_addr_b, io::Error::last_os_error());
        return;
    }
}



pub unsafe fn handle_hook_func(h_proc: HANDLE, func_hook: Hook, debug_event: DEBUG_EVENT, c_dbg: &mut bool) {
    let h_thread = OpenThread(THREAD_GET_CONTEXT | THREAD_SET_CONTEXT, FALSE, debug_event.dwThreadId);
    if !h_thread.is_null() {
        match NT_HEADER {
            Some(NtHeaders::Headers64(_)) => {
                let mut ctx: CONTEXT = mem::zeroed();
                ctx.ContextFlags = CONTEXT_ALL;
                if GetThreadContext(h_thread, &mut ctx) == 0 {
                    eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> failed to get thread context : {}", io::Error::last_os_error());
                    *c_dbg = false;
                    return;
                }
                let addr_target = func_hook.replacen + BASE_ADDR;
                ctx.Rip = addr_target;
                if SetThreadContext(h_thread, &ctx) == 0 {
                    eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> error when setting thread context : {}", io::Error::last_os_error());
                    TerminateProcess(h_proc, 0);
                    *c_dbg = false;
                } else {
                    println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> the program execution flow has been redirected to the address {:#x}", addr_target);
                }
            }
            Some(NtHeaders::Headers32(_)) => {
                let mut ctx: WOW64_CONTEXT = mem::zeroed();
                ctx.ContextFlags = WOW64_CONTEXT_ALL;
                if Wow64GetThreadContext(h_thread, &mut ctx) == 0 {
                    eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to get thread context : {}", io::Error::last_os_error());
                    TerminateProcess(h_proc, 0);
                    *c_dbg = false;
                } else {
                    let addr_target = func_hook.replacen + BASE_ADDR;
                    ctx.Eip = addr_target as u32;
                    if Wow64SetThreadContext(h_thread, &ctx) == 0 {
                        eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> error when setting thread context : {}", io::Error::last_os_error());
                        TerminateProcess(h_proc, 0);
                        *c_dbg = false;
                    } else {
                        println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> the program execution flow has been redirected to the address {:#x}", addr_target);
                    }
                }
            }
            None => {}
        }
    } else {
        eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> Failed to open thread: {}", io::Error::last_os_error());
        *c_dbg = false;
    }
}
