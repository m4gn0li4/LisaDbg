use std::{io, mem};
use winapi::shared::minwindef::{FALSE, LPVOID};
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::{ReadProcessMemory};
use winapi::um::minwinbase::DEBUG_EVENT;
use winapi::um::processthreadsapi::{GetThreadContext, OpenThread, SetThreadContext, TerminateProcess};
use winapi::um::winnt::{CONTEXT, CONTEXT_ALL, HANDLE, THREAD_GET_CONTEXT, THREAD_SET_CONTEXT};
use crate::dbg::{BASE_ADDR, dbg_cmd, memory, ST_BEGIN_INFO, StBeginFunc};
use crate::dbg::hook::Hook;
use crate::pefile::function;
use crate::pefile::function::Verifie;
use crate::log::*;




unsafe fn set_f(process_handle: HANDLE, ctx: CONTEXT, b_addr: u64) {
    let mut ret_addr: u64 = mem::zeroed();
    if ReadProcessMemory(process_handle, ctx.Rsp as LPVOID, &mut ret_addr as *mut _ as LPVOID, 8, &mut 0) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to read memory at address {:#x} : {}", ctx.Rsp, io::Error::last_os_error());
    }else {
        ST_BEGIN_INFO.sp = ctx.Rsp;
        ST_BEGIN_INFO.ret_addr = ret_addr;
        ST_BEGIN_INFO.begin_func_addr = b_addr;
    }
}



pub unsafe fn handle_stret(process_handle: HANDLE, debug_event: DEBUG_EVENT, breakpoint_addr: u64, continue_dbg: &mut bool, single_step: &mut bool) {
    println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Breakpoint hit at address: {:#x}", breakpoint_addr);
    memory::restore_byte_of_brkpt(process_handle, breakpoint_addr);
    memory::set_breakpoint_in_ret_func(process_handle, breakpoint_addr - BASE_ADDR);
    let h_thread = OpenThread(THREAD_GET_CONTEXT | THREAD_SET_CONTEXT, FALSE, debug_event.dwThreadId);
    if !h_thread.is_null() {
        let mut ctx = mem::zeroed::<CONTEXT>();
        ctx.ContextFlags = CONTEXT_ALL;
        if GetThreadContext(h_thread, &mut ctx) != 0 {
            set_f(process_handle, ctx, breakpoint_addr);
            ctx.Rip -= 1;
            if SetThreadContext(h_thread, &ctx) == 0 {
                eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> error when setting thread context : {}", io::Error::last_os_error())
            }else {
                *single_step = true;
            }
        } else {
            eprint!("[{ERR_COLOR}Critical{RESET_COLOR}]");
            eprintln!(" -> failed to get thread context : {}", io::Error::last_os_error());
            TerminateProcess(process_handle, 0);
            *continue_dbg = false
        }
        CloseHandle(h_thread);
    } else {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to open thread: {}", io::Error::last_os_error());
        TerminateProcess(process_handle, 0);
        *continue_dbg = false
    }
}




pub unsafe fn handle_br(process_handle: HANDLE, debug_event: DEBUG_EVENT, breakpoint_addr: u64, continue_dbg: &mut bool, single_step: &mut bool) {
    println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Breakpoint hit at address: {:#x}", breakpoint_addr);
    memory::restore_byte_of_brkpt(process_handle, breakpoint_addr);
    if ST_BEGIN_INFO.begin_func_addr != 0 && !function::FUNC_INFO.clone().is_in_func(ST_BEGIN_INFO.begin_func_addr, breakpoint_addr){
        ST_BEGIN_INFO = StBeginFunc::default();
    }
    let h_thread = OpenThread(THREAD_GET_CONTEXT | THREAD_SET_CONTEXT, FALSE, debug_event.dwThreadId);
    if !h_thread.is_null() {
        let mut ctx = mem::zeroed::<CONTEXT>();
        ctx.ContextFlags = CONTEXT_ALL;
        if GetThreadContext(h_thread, &mut ctx) != 0 {
            ctx.Rip -= 1;
            dbg_cmd::cmd_wait(&mut ctx, process_handle, continue_dbg);
            if SetThreadContext(h_thread, &ctx) == 0 {
                eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> error when setting thread context : {}", io::Error::last_os_error())
            }
            *single_step = true;
        } else {
            eprint!("[{ERR_COLOR}Critical{RESET_COLOR}]");
            eprintln!("-> failed to get thread context : {}", io::Error::last_os_error());
        }
        CloseHandle(h_thread);
    } else {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to open thread: {}", io::Error::last_os_error());
    }
}


pub unsafe fn handle_hook_func(process_handle: HANDLE, func_hook: Hook, debug_event: DEBUG_EVENT, continue_debug: &mut bool) {
    let h_thread = OpenThread(THREAD_GET_CONTEXT|THREAD_SET_CONTEXT, FALSE, debug_event.dwThreadId);
    if !h_thread.is_null() {
        let mut ctx: CONTEXT = mem::zeroed();
        ctx.ContextFlags = CONTEXT_ALL;
        if GetThreadContext(h_thread, &mut ctx) != 0{
            let addr_target = func_hook.replacen + BASE_ADDR;
            ctx.Rip = addr_target;
            if SetThreadContext(h_thread, &ctx) == 0 {
                eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> error setting rip register to address {:#x} : {}", ctx.Rip, io::Error::last_os_error());
                memory::restore_byte_of_brkpt(process_handle, func_hook.target + BASE_ADDR);
                ctx.Rip -= 1;
                if SetThreadContext(h_thread, &ctx) == 0 {
                    eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> error when setting thread context : {}", io::Error::last_os_error());
                    TerminateProcess(process_handle, 0);
                    *continue_debug = true;
                }
            }else {
                println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> the program execution flow has been redirected to the address {:#x}", addr_target);
            }
        }else {
            eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> failed to get thread context : {}", io::Error::last_os_error());
        }
    }else {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to open thread: {}", io::Error::last_os_error());
    }
}


