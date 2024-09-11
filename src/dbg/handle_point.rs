use std::{io, mem};
use std::ptr::addr_of;
use winapi::shared::minwindef::FALSE;
use winapi::um::handleapi::CloseHandle;
use winapi::um::minwinbase::DEBUG_EVENT;
use winapi::um::processthreadsapi::{GetThreadContext, OpenThread, SetThreadContext, TerminateProcess};
use winapi::um::winbase::{Wow64GetThreadContext, Wow64SetThreadContext};
use winapi::um::winnt::{CONTEXT, CONTEXT_ALL, HANDLE, THREAD_GET_CONTEXT, THREAD_SET_CONTEXT, WOW64_CONTEXT, WOW64_CONTEXT_ALL};
use crate::dbg::{BASE_ADDR, dbg_cmd, memory};
use crate::pefile::NT_HEADER;
use crate::utils::*;
use crate::{pefile, symbol};
use crate::command::hook::Hook;

pub unsafe fn handle_br(process_handle: HANDLE, debug_event: DEBUG_EVENT, breakpoint_addr: u64, continue_dbg: &mut bool) {
    println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Breakpoint hit at address: {:#x}", breakpoint_addr);
    memory::breakpoint::restore_byte_of_brkpt(process_handle, breakpoint_addr);

    let h_thread = OpenThread(THREAD_GET_CONTEXT | THREAD_SET_CONTEXT, FALSE, debug_event.dwThreadId);
    if h_thread.is_null() {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to open thread: {}", io::Error::last_os_error());
        return;
    }
    match NT_HEADER {
        Some(pefile::NtHeaders::Headers64(_)) => {
            let mut ctx = mem::zeroed::<CONTEXT>();
            ctx.ContextFlags = CONTEXT_ALL;
            if GetThreadContext(h_thread, &mut ctx) != 0 {
                ctx.Rip -= 1;
                dbg_cmd::x64::cmd_wait(&mut ctx, process_handle,h_thread, continue_dbg);
                if SetThreadContext(h_thread, &ctx) == 0 {
                    eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> error when setting thread context: {}", io::Error::last_os_error());
                }
            } else {
                eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> failed to get thread context: {}", io::Error::last_os_error());
            }
        }
        Some(pefile::NtHeaders::Headers32(_)) => {
            let mut ctx = mem::zeroed::<WOW64_CONTEXT>();
            ctx.ContextFlags = WOW64_CONTEXT_ALL;
            if Wow64GetThreadContext(h_thread, &mut ctx) != 0 {
                ctx.Eip -= 1;
                dbg_cmd::x32::cmd_wait32(&mut ctx, process_handle, h_thread, continue_dbg);
                if Wow64SetThreadContext(h_thread, &ctx) == 0 {
                    eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> error when setting thread context: {}", io::Error::last_os_error());
                }
            } else {
                eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> failed to get thread context: {}", io::Error::last_os_error());
            }
        }
        None => eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> how is it possible ? hmm suspicious that"),
    }
    CloseHandle(h_thread);
}




pub unsafe fn handle_hook_func(process_handle: HANDLE, func_hook: Hook, debug_event: DEBUG_EVENT, continue_debug: &mut bool) {
    let h_thread = OpenThread(THREAD_GET_CONTEXT|THREAD_SET_CONTEXT, FALSE, debug_event.dwThreadId);
    if !h_thread.is_null() {
        match NT_HEADER {
            Some(pefile::NtHeaders::Headers64(_)) => {
                let mut ctx: CONTEXT = mem::zeroed();
                ctx.ContextFlags = CONTEXT_ALL;
                if GetThreadContext(h_thread, &mut ctx) != 0{
                    let addr_target = func_hook.replacen + BASE_ADDR;
                    ctx.Rip = addr_target;
                    if SetThreadContext(h_thread, &ctx) == 0 {
                        eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> error when setting thread context : {}", io::Error::last_os_error());
                        TerminateProcess(process_handle, 0);
                        *continue_debug = false;
                    }else {
                        println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> the program execution flow has been redirected to the address {:#x}", addr_target);
                    }
                }else {
                    eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> failed to get thread context : {}", io::Error::last_os_error());
                }
            }
            Some(pefile::NtHeaders::Headers32(_)) => {
                let mut ctx: WOW64_CONTEXT = mem::zeroed();
                ctx.ContextFlags = WOW64_CONTEXT_ALL;
                if Wow64GetThreadContext(h_thread, &mut ctx) == 0 {
                    eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to get thread context : {}", io::Error::last_os_error());
                    TerminateProcess(process_handle, 0);
                    *continue_debug = false;
                }else {
                    let addr_target = func_hook.replacen + BASE_ADDR;
                    ctx.Eip = addr_target as u32;
                    if Wow64SetThreadContext(h_thread, &ctx) == 0 {
                        eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> error when setting thread context : {}", io::Error::last_os_error());
                        TerminateProcess(process_handle, 0);
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



pub unsafe fn handle_watchpoint(debug_event: DEBUG_EVENT, process_handle: HANDLE, continue_dbg: &mut bool) {
    let h_thread = OpenThread(THREAD_GET_CONTEXT | THREAD_SET_CONTEXT, FALSE, debug_event.dwThreadId);
    if !h_thread.is_null() {
        match &*addr_of!(NT_HEADER) {
            Some(pefile::NtHeaders::Headers64(_)) => {
                let mut ctx: CONTEXT = mem::zeroed();
                ctx.ContextFlags = CONTEXT_ALL;
                if GetThreadContext(h_thread, &mut ctx) == 0 {
                    eprint!("[{ERR_COLOR}Error{RESET_COLOR}]");
                    eprintln!(" -> failed to get thread context : {}", io::Error::last_os_error());
                    return;
                }else {
                    let access_addr = if ctx.Dr6 & 0b0001 != 0 {
                        ctx.Dr0
                    } else if ctx.Dr6 & 0b0010 != 0 {
                        ctx.Dr1
                    } else if ctx.Dr6 & 0b0100 != 0 {
                        ctx.Dr2
                    } else if ctx.Dr6 & 0b1000 != 0 {
                        ctx.Dr3
                    } else {
                        0
                    };
                    let name;
                    if let Some(sym) = symbol::SYMBOLS_V.symbol_file.iter().find(|s|s.offset + BASE_ADDR as i64 == access_addr as i64) {
                        name = format!("of {} - {:#x}", sym.name, access_addr);
                    }else {
                        name = format!("{:#x}", access_addr);
                    }
                    let except_addr = debug_event.u.Exception().ExceptionRecord.ExceptionAddress;
                    println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> except address {:#x}, there was access to the address {name}", except_addr as u64);
                    dbg_cmd::x64::cmd_wait(&mut ctx, process_handle, h_thread, continue_dbg);
                    if SetThreadContext(h_thread, &ctx) == 0 {
                        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to set ctx of thread, all modification of ctx is useless : {}", io::Error::last_os_error());
                    }
                }
            }
            Some(pefile::NtHeaders::Headers32(_)) => {
                let mut ctx: WOW64_CONTEXT = mem::zeroed();
                ctx.ContextFlags = WOW64_CONTEXT_ALL;
                if Wow64GetThreadContext(h_thread, &mut ctx) == 0 {
                    eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to get thread context : {}", io::Error::last_os_error());
                    return;
                }else {
                    let access_addr = if ctx.Dr6 & 0b0001 != 0 {
                        ctx.Dr0
                    } else if ctx.Dr6 & 0b0010 != 0 {
                        ctx.Dr1
                    } else if ctx.Dr6 & 0b0100 != 0 {
                        ctx.Dr2
                    } else if ctx.Dr6 & 0b1000 != 0 {
                        ctx.Dr3
                    } else {
                        0
                    };
                    let name;
                    if let Some(sym) = symbol::SYMBOLS_V.symbol_file.iter().find(|s|(s.offset + BASE_ADDR as i64) == access_addr as i64) {
                        name = format!("of {} - {:#x}", sym.name, access_addr);
                    }else {
                        name = format!("{:#x}", access_addr);
                    }
                    let except_addr = debug_event.u.Exception().ExceptionRecord.ExceptionAddress;
                    println!("[{VALID_COLOR}Debug{RESET_COLOR}] -> except address :{:#x} watchpoint caused by the instruction at the address {name}", except_addr as u64);
                    dbg_cmd::x32::cmd_wait32(&mut ctx, process_handle, h_thread, continue_dbg);
                    if Wow64SetThreadContext(h_thread, &ctx) == 0 {
                        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to set ctx of thread, all modification of ctx is useless : {}", io::Error::last_os_error());
                    }
                }
            }
            None => {}
        }
    }else {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to get the handle of thread : {}", io::Error::last_os_error());
    }
}
