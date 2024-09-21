use std::{io, mem};
use std::ptr::addr_of;
use winapi::shared::minwindef::FALSE;
use winapi::um::handleapi::CloseHandle;
use winapi::um::minwinbase::DEBUG_EVENT;
use winapi::um::processthreadsapi::{GetThreadContext, OpenThread, SetThreadContext};
use winapi::um::winbase::{Wow64GetThreadContext, Wow64SetThreadContext};
use winapi::um::winnt::*;
use crate::command::watchpoint::Watchpts;
use crate::{pefile, symbol, ALL_ELM};
use crate::dbg::{dbg_cmd, RealAddr, BASE_ADDR};
use crate::pefile::NT_HEADER;
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



pub unsafe fn set_watchpoint(debug_event: DEBUG_EVENT, _h_proc: HANDLE) {
    let h_thread = OpenThread(THREAD_ALL_ACCESS, FALSE, debug_event.dwThreadId);
    if !h_thread.is_null() {
        match &*addr_of!(NT_HEADER) {
            Some(nt_head) => match nt_head{
                pefile::NtHeaders::Headers32(_) => {
                    let mut ctx: WOW64_CONTEXT = mem::zeroed();
                    ctx.ContextFlags = WOW64_CONTEXT_DEBUG_REGISTERS;
                    if Wow64GetThreadContext(h_thread, &mut ctx) == 0{
                        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to get context for set watchpoint, all watchpoint is useless : {}", io::Error::last_os_error());
                        return;
                    }
                    for (i, watchpts) in ALL_ELM.watchpts.iter().enumerate() {
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
                    let mut ctx: CONTEXT = mem::zeroed();
                    ctx.ContextFlags = CONTEXT_DEBUG_REGISTERS;
                    if GetThreadContext(h_thread, &mut ctx) == 0{
                        eprint!("[{ERR_COLOR}Error{RESET_COLOR}]");
                        eprintln!(" -> failed to get context for set watchpoint, all watchpoint is useless : {}", io::Error::last_os_error());
                        return;
                    }
                    for (i, watchpts) in ALL_ELM.watchpts.iter().enumerate() {
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






pub unsafe fn handle_watchpoint(debug_event: DEBUG_EVENT, h_proc: HANDLE, continue_dbg: &mut bool) {
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
                    dbg_cmd::x64::cmd_wait(&mut ctx, h_proc, h_thread, continue_dbg);
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
                    dbg_cmd::x32::cmd_wait32(&mut ctx, h_proc, h_thread, continue_dbg);
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