use std::{io, ptr};
use winapi::shared::minwindef::FALSE;
use winapi::um::handleapi::CloseHandle;
use winapi::um::minwinbase::DEBUG_EVENT;
use winapi::um::processthreadsapi::{GetThreadContext, OpenThread, SetThreadContext};
use winapi::um::winbase::Wow64SetThreadContext;
use winapi::um::winnt::*;
use crate::command::watchpoint::Watchpts;
use crate::{OPTION, pefile};
use crate::dbg::RealAddr;
use crate::utils::*;


pub fn clear_dreg(ctx: &mut CONTEXT, reg_index: usize) {
    match reg_index {
        0 => ctx.Dr0 = 0,
        1 => ctx.Dr1 = 0,
        2 => ctx.Dr2 = 0,
        3 => ctx.Dr3 = 0,
        _ => {},
    }
    let mask = !(1 << (reg_index * 2));
    ctx.Dr7 &= mask;
    ctx.Dr7 &= !(0b1111 << (16 + reg_index * 4));
    ctx.Dr7 &= !(0b11 << (18 + reg_index * 4));
}




pub fn set_dreg(ctx: &mut CONTEXT, watch: &Watchpts, reg_index: usize) {
    match reg_index {
        0 => ctx.Dr0 = watch.real_addr64(*ctx),
        1 => ctx.Dr1 = watch.real_addr64(*ctx),
        2 => ctx.Dr2 = watch.real_addr64(*ctx),
        3 => ctx.Dr3 = watch.real_addr64(*ctx),
        _ => {},
    }

    ctx.Dr7 |= 1 << (reg_index * 2);
    let access_bits = watch.acces_type_to_bits() as u64;
    ctx.Dr7 &= !(0b11 << (16 + reg_index * 4));
    ctx.Dr7 |= access_bits << (16 + reg_index * 4);

    let size_bits = match watch.memory_size {
        1 => 0b00,
        2 => 0b01,
        4 => 0b11,
        8 => 0b10,
        _ => {
            println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> invalid memory size for watchpoint {reg_index}, default size = 1");
            0b00
        }
    };
    ctx.Dr7 &= !(0b11 << (18 + reg_index * 4));
    ctx.Dr7 |= size_bits << (18 + reg_index * 4);
}




fn set_watchpoint32(ctx: &mut WOW64_CONTEXT, watch: &Watchpts, reg_index: usize) {
    match reg_index {
        0 => ctx.Dr0 = watch.real_addr32(*ctx),
        1 => ctx.Dr1 = watch.real_addr32(*ctx),
        2 => ctx.Dr2 = watch.real_addr32(*ctx),
        3 => ctx.Dr3 = watch.real_addr32(*ctx),
        _ => {},
    }

    ctx.Dr7 |= 1 << (reg_index * 2);
    let access_bits = watch.acces_type_to_bits();
    ctx.Dr7 &= !(0b11 << (16 + reg_index * 4));
    ctx.Dr7 |= access_bits << (16 + reg_index * 4);

    let size_bits = match watch.memory_size {
        1 => 0b00,
        2 => 0b01,
        4 => 0b11,
        8 => 0b10,
        _ => {
            eprintln!("[{DBG_COLOR}Debug{RESET_COLOR}] -> invalid memory size: default size = 1");
            0b00
        }
    };
    ctx.Dr7 &= !(0b11 << (18 + reg_index * 4));
    ctx.Dr7 |= size_bits << (18 + reg_index * 4);
}



pub unsafe fn set_watchpoint(debug_event: DEBUG_EVENT, _process_handle: HANDLE) {
    let h_thread = OpenThread(THREAD_ALL_ACCESS, FALSE, debug_event.dwThreadId);
    if !h_thread.is_null() {
        match &*ptr::addr_of!(pefile::NT_HEADER) {
            Some(nt_head) => match nt_head{
                pefile::NtHeaders::Headers32(_) => {
                    let mut ctx: WOW64_CONTEXT = std::mem::zeroed();
                    ctx.ContextFlags = WOW64_CONTEXT_DEBUG_REGISTERS;
                    if winapi::um::winbase::Wow64GetThreadContext(h_thread, &mut ctx) == 0{
                        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to get context for set watchpoint, all watchpoint is useless : {}", io::Error::last_os_error());
                        return;
                    }
                    for (i, watchpts) in OPTION.watchpts.iter().enumerate() {
                        set_watchpoint32(&mut ctx, watchpts, i);
                    }
                    if Wow64SetThreadContext(h_thread, &mut ctx) == 0 {
                        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to set context for set watchpoint, all watchpoints are useless: {}", io::Error::last_os_error());
                        return;
                    } else {
                        println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> all watchpoints set successfully");
                    }
                }
                pefile::NtHeaders::Headers64(_) => {
                    let mut ctx: CONTEXT = std::mem::zeroed();
                    ctx.ContextFlags = CONTEXT_DEBUG_REGISTERS;
                    if GetThreadContext(h_thread, &mut ctx) == 0{
                        eprint!("[{ERR_COLOR}Error{RESET_COLOR}]");
                        eprintln!(" -> failed to get context for set watchpoint, all watchpoint is useless : {}", io::Error::last_os_error());
                        return;
                    }
                    for (i, watchpts) in OPTION.watchpts.iter().enumerate() {
                        set_dreg(&mut ctx, watchpts, i);
                    }
                    if SetThreadContext(h_thread, &mut ctx) == 0 {
                        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to set context for set watchpoint, all watchpoints are useless: {}", io::Error::last_os_error());
                        return;
                    } else {
                        println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> all watchpoints set successfully");
                    }
                }
            }
            None => {}
        }
        CloseHandle(h_thread);
    }else {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}]");
        eprintln!(" -> Failed to open thread : {}", io::Error::last_os_error());
    }
}