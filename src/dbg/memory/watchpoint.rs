use crate::command::watchpoint::Watchpts;
use crate::dbg::{dbg_cmd, RealAddr, BASE_ADDR};
use crate::pefile::NT_HEADER;
use crate::utils::*;
use crate::{pefile, symbol, ALL_ELM};
use std::ops::{BitAnd, BitOrAssign};
use std::ptr::addr_of;
use std::{io, mem};
use winapi::shared::minwindef::FALSE;
use winapi::um::handleapi::CloseHandle;
use winapi::um::minwinbase::DEBUG_EVENT;
use winapi::um::processthreadsapi::{GetThreadContext, OpenThread, SetThreadContext};
use winapi::um::winbase::{Wow64GetThreadContext, Wow64SetThreadContext};
use winapi::um::winnt::*;

pub fn clear_dreg(ctx: &mut CONTEXT, reg_index: usize) {
    match reg_index {
        0 => ctx.Dr0 = 0,
        1 => ctx.Dr1 = 0,
        2 => ctx.Dr2 = 0,
        3 => ctx.Dr3 = 0,
        _ => {}
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
        _ => {}
    }
    set_dr7::<u64>(&mut ctx.Dr7, reg_index, watch);
}

fn set_dr7<T: NumConvert + std::ops::BitAndAssign + BitOrAssign>(dr7: &mut T, reg_index: usize, watch: &Watchpts) {
    *dr7 |= T::from_u64(1 << (reg_index * 2));
    let access_bits = watch.acces_type_to_bits();
    *dr7 &= T::from_u64(!(0b11 << (16 + reg_index * 4)));
    *dr7 |= T::from_u64((access_bits << (16 + reg_index * 4)) as u64);

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
    *dr7 &= T::from_u64(!(0b11 << (18 + reg_index * 4)));
    *dr7 |= T::from_u64(size_bits << (18 + reg_index * 4));
}

fn set_watchpoint32(ctx: &mut WOW64_CONTEXT, watch: &Watchpts, reg_index: usize) {
    match reg_index {
        0 => ctx.Dr0 = watch.real_addr32(*ctx),
        1 => ctx.Dr1 = watch.real_addr32(*ctx),
        2 => ctx.Dr2 = watch.real_addr32(*ctx),
        3 => ctx.Dr3 = watch.real_addr32(*ctx),
        _ => {}
    }
    set_dr7::<u32>(&mut ctx.Dr7, reg_index, watch);
}


pub unsafe fn set_watchpoint(debug_event: DEBUG_EVENT, _h_proc: HANDLE) {
    let h_thread = OpenThread(THREAD_ALL_ACCESS, FALSE, debug_event.dwThreadId);
    if !h_thread.is_null() {
        match &*addr_of!(NT_HEADER) {
            Some(nt_head) => match nt_head {
                pefile::NtHeaders::Headers32(_) => {
                    let mut ctx: WOW64_CONTEXT = mem::zeroed();
                    ctx.ContextFlags = WOW64_CONTEXT_ALL;
                    if Wow64GetThreadContext(h_thread, &mut ctx) == 0 {
                        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to get context for set watchpoint, all watchpoint is useless : {}", io::Error::last_os_error());
                        return;
                    }
                    for (i, watchpts) in ALL_ELM.watchpts.iter().enumerate() {
                        set_watchpoint32(&mut ctx, watchpts, i);
                        println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> activation of watchpoint {i} monitoring on register dr{i} : {:#x}", BASE_ADDR);
                    }
                    if Wow64SetThreadContext(h_thread, &mut ctx) == 0 {
                        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to set context for set watchpoint, all watchpoints are useless: {}", io::Error::last_os_error());
                        return;
                    }
                }
                pefile::NtHeaders::Headers64(_) => {
                    let mut ctx: CONTEXT = mem::zeroed();
                    ctx.ContextFlags = CONTEXT_ALL;
                    if GetThreadContext(h_thread, &mut ctx) == 0 {
                        eprint!("[{ERR_COLOR}Error{RESET_COLOR}]");
                        eprintln!(" -> failed to get context for set watchpoint, all watchpoint is useless : {}", io::Error::last_os_error());
                        return;
                    }
                    for (i, watchpts) in ALL_ELM.watchpts.iter().enumerate() {
                        set_dreg(&mut ctx, watchpts, i);
                        println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> activation of watchpoint {i} monitoring on register dr{i}");
                    }
                    if SetThreadContext(h_thread, &mut ctx) == 0 {
                        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to set context for set watchpoint, all watchpoints are useless: {}", io::Error::last_os_error());
                        return;
                    }
                }
            },
            None => {}
        }
        CloseHandle(h_thread);
    } else {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}]");
        eprintln!(" -> Failed to open thread : {}", io::Error::last_os_error());
    }
}

fn get_acc_addr<T: NumConvert + BitAnd + PartialEq + Copy>(dr6: T, dr0: T, dr1: T, dr2: T, dr3: T) -> T
where
    <T as BitAnd>::Output: PartialEq<T>,
{
    let zer = T::from_u64(0);
    if dr6 & T::from_u64(0b0001) != zer {
        dr0
    } else if dr6 & T::from_u64(0b0010) != zer {
        dr1
    } else if dr6 & T::from_u64(0b0100) != zer {
        dr2
    } else if dr6 & T::from_u64(0b1000) != zer {
        dr3
    } else {
        T::from_u64(0)
    }
}

unsafe fn get_b(access_addr: u64) -> String{
    if let Some(sym) = symbol::SYMBOLS_V.symbol_file.iter().find(|s| s.offset + BASE_ADDR as i64 == access_addr as i64) {
        format!("of {} - {:#x}", sym.name, access_addr)
    } else {
        format!("{:#x}", access_addr)
    }
}

pub unsafe fn handle_watchpoint64(debug_event: DEBUG_EVENT, h_proc: HANDLE, h_thread: &mut HANDLE, ctx: &mut CONTEXT, continue_dbg: &mut bool) {
    let access_addr = get_acc_addr::<u64>(ctx.Dr6, ctx.Dr0, ctx.Dr1, ctx.Dr2, ctx.Dr3);
    let name = get_b(access_addr);
    let except_addr = debug_event.u.Exception().ExceptionRecord.ExceptionAddress;
    println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> except address {:#x}, there was access to the address {name}", except_addr as u64);
    dbg_cmd::x64::cmd_wait(ctx, h_proc, h_thread, continue_dbg);
}



pub unsafe fn handle_watchpoint32(debug_event: DEBUG_EVENT, h_proc: HANDLE, h_thread: &mut HANDLE, ctx: &mut WOW64_CONTEXT, continue_dbg: &mut bool) {
    let access_addr = get_acc_addr::<u32>(ctx.Dr6, ctx.Dr0, ctx.Dr1, ctx.Dr2, ctx.Dr3);
    let name = get_b(access_addr as u64);
    let except_addr = debug_event.u.Exception().ExceptionRecord.ExceptionAddress;
    println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> except address {:#x}, there was access to the address {name}", except_addr as u64);
    dbg_cmd::x32::cmd_wait32(ctx, h_proc, h_thread, continue_dbg);
}