use std::{io, mem};
use winapi::shared::minwindef::{FALSE, LPVOID};
use winapi::um::memoryapi::WriteProcessMemory;
use winapi::um::minwinbase::DEBUG_EVENT;
use winapi::um::processthreadsapi::{GetThreadContext, OpenThread, SetThreadContext, TerminateProcess};
use winapi::um::winbase::{Wow64GetThreadContext, Wow64SetThreadContext};
use winapi::um::winnt::{CONTEXT, CONTEXT_ALL, HANDLE, THREAD_ALL_ACCESS, THREAD_GET_CONTEXT, THREAD_SET_CONTEXT, WOW64_CONTEXT, WOW64_CONTEXT_ALL};
use crate::dbg::BASE_ADDR;
use crate::pefile::{NtHeaders, NT_HEADER};
use crate::utils::*;
use crate::cli::AfterB;
use crate::command::hook::Hook;
use crate::dbg::memory::breakpoint::restore_byte_of_brkpt;



pub unsafe fn handle_after_b(h_proc: HANDLE, ab: AfterB, debug_event: DEBUG_EVENT) {
    restore_byte_of_brkpt(h_proc, ab.after_b);
    let h_thread = OpenThread(THREAD_ALL_ACCESS, 0, debug_event.dwThreadId);
    if h_thread.is_null() {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to open thread : {}{RESET_COLOR}", io::Error::last_os_error());
        return;
    }
    match NT_HEADER.unwrap() {
        NtHeaders::Headers32(_) => {
            let mut ctx: WOW64_CONTEXT = mem::zeroed();
            ctx.ContextFlags = WOW64_CONTEXT_ALL;
            if Wow64GetThreadContext(h_thread, &mut ctx) == 0 {
                eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to get thread context: {}", io::Error::last_os_error());
                return;
            }
            ctx.Eip -= 1;
            if Wow64SetThreadContext(h_thread, &ctx) == 0 {
                eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to set thread context: {}", io::Error::last_os_error());
                return;
            }
        }
        NtHeaders::Headers64(_) => {
            let mut ctx: CONTEXT = mem::zeroed();
            ctx.ContextFlags = CONTEXT_ALL;
            if GetThreadContext(h_thread, &mut ctx) == 0 {
                eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to get thread context: {}", io::Error::last_os_error());
                return;
            }
            ctx.Rip -= 1;
            if SetThreadContext(h_thread, &ctx) == 0 {
                eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to set thread context: {}", io::Error::last_os_error());
                return;
            }
        }
    }
    if WriteProcessMemory(h_proc, ab.last_addr_b as LPVOID, &0xccu8 as *const u8 as LPVOID, 1, &mut 0) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to write memory at address {:#x} : {}", ab.last_addr_b, io::Error::last_os_error());
        return;
    }
}


pub unsafe fn handle_hook_func(h_proc: HANDLE, func_hook: Hook, debug_event: DEBUG_EVENT, continue_debug: &mut bool) {
    let h_thread = OpenThread(THREAD_GET_CONTEXT|THREAD_SET_CONTEXT, FALSE, debug_event.dwThreadId);
    if !h_thread.is_null() {
        match NT_HEADER {
            Some(NtHeaders::Headers64(_)) => {
                let mut ctx: CONTEXT = mem::zeroed();
                ctx.ContextFlags = CONTEXT_ALL;
                if GetThreadContext(h_thread, &mut ctx) != 0{
                    let addr_target = func_hook.replacen + BASE_ADDR;
                    ctx.Rip = addr_target;
                    if SetThreadContext(h_thread, &ctx) == 0 {
                        eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> error when setting thread context : {}", io::Error::last_os_error());
                        TerminateProcess(h_proc, 0);
                        *continue_debug = false;
                    }else {
                        println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> the program execution flow has been redirected to the address {:#x}", addr_target);
                    }
                }else {
                    eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> failed to get thread context : {}", io::Error::last_os_error());
                }
            }
            Some(NtHeaders::Headers32(_)) => {
                let mut ctx: WOW64_CONTEXT = mem::zeroed();
                ctx.ContextFlags = WOW64_CONTEXT_ALL;
                if Wow64GetThreadContext(h_thread, &mut ctx) == 0 {
                    eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to get thread context : {}", io::Error::last_os_error());
                    TerminateProcess(h_proc, 0);
                    *continue_debug = false;
                }else {
                    let addr_target = func_hook.replacen + BASE_ADDR;
                    ctx.Eip = addr_target as u32;
                    if Wow64SetThreadContext(h_thread, &ctx) == 0 {
                        eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> error when setting thread context : {}", io::Error::last_os_error());
                        TerminateProcess(h_proc, 0);
                        *continue_debug = false;
                    }else {
                        println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> the program execution flow has been redirected to the address {:#x}", addr_target);
                    }
                }
            }
            None => {}
        }
    }else {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to open thread: {}", io::Error::last_os_error());
    }
}



