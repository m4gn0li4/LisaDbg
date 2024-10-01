use crate::dbg;
use crate::dbg::dbg_cmd::usages::USAGE_DBG_T;
use crate::pefile::{NtHeaders, NT_HEADER};
use crate::utils::{str_to, DBG_COLOR, ERR_COLOR, MAGENTA, RESET_COLOR, VALID_COLOR, VALUE_COLOR};
use ntapi::ntpsapi::{NtQueryInformationThread, THREAD_BASIC_INFORMATION};
use std::{io, ptr};
use winapi::shared::minwindef::LPVOID;
use winapi::um::handleapi::CloseHandle;
use winapi::um::processthreadsapi::{GetThreadContext, OpenThread, SetThreadContext};
use winapi::um::winbase::Wow64GetThreadContext;
use winapi::um::winnt::{
    CONTEXT, CONTEXT_ALL, HANDLE, THREAD_ALL_ACCESS, WOW64_CONTEXT, WOW64_CONTEXT_ALL,
};

pub fn change_dbg_thread(linev: &[&str], ctx: *mut CONTEXT, h_proc: HANDLE, h_thread1: &mut HANDLE, addr_func: &mut u64) {
    if linev.len() < 2 {
        println!("{USAGE_DBG_T}");
        return;
    }

    let tip = match str_to::<u32>(linev[1]) {
        Ok(tid) => tid,
        Err(e) => {
            eprintln!("{ERR_COLOR}failed to parse thread id : {e}{RESET_COLOR}");
            return;
        }
    };

    unsafe {
        if SetThreadContext(*h_thread1, ctx) == 0 {
            eprintln!(
                "{ERR_COLOR}failed to set thread context: {}{RESET_COLOR}",
                io::Error::last_os_error()
            );
            return;
        }

        let h_thread = OpenThread(THREAD_ALL_ACCESS, 0, tip);
        if h_thread.is_null() {
            eprintln!(
                "{ERR_COLOR}failed to open thread : {}{RESET_COLOR}",
                io::Error::last_os_error()
            );
            return;
        }
        CloseHandle(*h_thread1);
        *h_thread1 = h_thread;
        match NT_HEADER {
            Some(NtHeaders::Headers64(_)) => {
                let mut new_ctx: CONTEXT = std::mem::zeroed();
                new_ctx.ContextFlags = CONTEXT_ALL;
                if GetThreadContext(h_thread, &mut new_ctx) == 0 {
                    eprintln!("{ERR_COLOR}failed to get thread context: {}{RESET_COLOR}", io::Error::last_os_error());
                    return;
                }
                *ctx = new_ctx;
                dbg::dbg_cmd::init_cm(*ctx, h_proc, h_thread, addr_func);
            }
            Some(NtHeaders::Headers32(_)) => {
                let mut new_ctx: WOW64_CONTEXT = std::mem::zeroed();
                new_ctx.ContextFlags = WOW64_CONTEXT_ALL;
                if Wow64GetThreadContext(h_thread, &mut new_ctx) == 0 {
                    eprintln!("{ERR_COLOR}failed to get thread context: {}{RESET_COLOR}", io::Error::last_os_error());
                    return;
                }
                *(ctx as *mut WOW64_CONTEXT) = new_ctx;
                let mut addr2 = *addr_func as u32;
                dbg::dbg_cmd::x32::init_cm(*(ctx as *mut WOW64_CONTEXT), h_proc, h_thread, &mut addr2);
                *addr_func = addr2 as u64;
            }
            _ => {}
        }
        println!("{VALID_COLOR}now you are on the thread {tip}{RESET_COLOR}");
    }
}

pub fn get_thread_now(h_thread: HANDLE) {
    unsafe {
        let tbi: THREAD_BASIC_INFORMATION = std::mem::zeroed();
        let ntstatus = NtQueryInformationThread(h_thread, ntapi::ntpsapi::ThreadBasicInformation, ptr::addr_of!(tbi) as LPVOID, size_of::<THREAD_BASIC_INFORMATION>() as u32, &mut 0);
        if ntstatus == 0 {
            println!("{}Thread: ", DBG_COLOR);
            println!("    {}Thread id : {}", MAGENTA, tbi.ClientId.UniqueThread as u64);
            println!("    {}Owner pid : {}", VALUE_COLOR, tbi.ClientId.UniqueProcess as u64);
            println!();
        } else {
            eprintln!("{ERR_COLOR}failed to query info of current thread to debug with ntstatus : {ntstatus}{RESET_COLOR}");
        }
    }
}
