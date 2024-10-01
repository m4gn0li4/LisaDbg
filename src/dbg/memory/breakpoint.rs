use crate::cli::AfterB;
use crate::dbg::memory::stack::{get_frame_st, get_frame_st32, get_real_frame, ST_FRAME};
use crate::dbg::{dbg_cmd, SaveInsn, BASE_ADDR, SAVEINSN};
use crate::pefile::{NtHeaders, NT_HEADER};
use crate::utils::*;
use crate::ALL_ELM;
use iced_x86::{Decoder, DecoderOptions, Instruction};
use std::{io, mem, ptr};
use winapi::shared::minwindef::{FALSE, LPVOID};
use winapi::shared::ntdef::HANDLE;
use winapi::um::dbghelp::SymCleanup;
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::*;
use winapi::um::minwinbase::DEBUG_EVENT;
use winapi::um::processthreadsapi::{GetThreadContext, OpenThread, SetThreadContext};
use winapi::um::winbase::{Wow64GetThreadContext, Wow64SetThreadContext};
use winapi::um::winnt::{
    CONTEXT, CONTEXT_ALL, PAGE_EXECUTE_READWRITE, THREAD_ALL_ACCESS, WOW64_CONTEXT,
    WOW64_CONTEXT_ALL,
};

pub unsafe fn restore_byte_of_brkpt(h_proc: HANDLE, b_addr: u64) {
    if let Some(pos) = SAVEINSN.iter().position(|s| s.addr == b_addr) {
        let insn = SAVEINSN.remove(pos);
        let mut old_protect = 0;
        let mut written = 0;
        if VirtualProtectEx(h_proc, insn.addr as LPVOID, 1, PAGE_EXECUTE_READWRITE, &mut old_protect) == 0 {
            eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> error when changing memory protection at address : {:#x}", b_addr);
            return;
        }
        if WriteProcessMemory(h_proc, insn.addr as LPVOID, ptr::addr_of!(insn.last_oc) as LPVOID, 1, &mut written) == 0 {
            eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> error when writing to memory at address : {:#x} : {}", b_addr, io::Error::last_os_error());
        } else {
            println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Restored original byte at address: {:#x}", b_addr);
        }
        if VirtualProtectEx(h_proc, insn.addr as LPVOID, 1, old_protect, &mut old_protect) == 0 {
            eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> error while restoring memory protection at address: {:#x}", b_addr)
        }
    }
}



pub unsafe fn handle_br(h_proc: winapi::um::winnt::HANDLE, debug_event: DEBUG_EVENT, b_addr: u64, continue_dbg: &mut bool, ) {
    println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Breakpoint hit at address: {:#x}", b_addr);
    restore_byte_of_brkpt(h_proc, b_addr);

    let mut h_thread = OpenThread(THREAD_ALL_ACCESS, FALSE, debug_event.dwThreadId);
    if h_thread.is_null() {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to open thread: {}", io::Error::last_os_error());
        return;
    }
    match NT_HEADER {
        Some(NtHeaders::Headers64(_)) => {
            let mut ctx = mem::zeroed::<CONTEXT>();
            ctx.ContextFlags = CONTEXT_ALL;
            if GetThreadContext(h_thread, &mut ctx) != 0 {
                ctx.Rip -= 1;
                dbg_cmd::x64::cmd_wait(&mut ctx, h_proc, &mut h_thread, continue_dbg);
                if SetThreadContext(h_thread, &ctx) == 0 {
                    eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> error when setting thread context: {}", io::Error::last_os_error());
                }
            } else {
                eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> failed to get thread context: {}", io::Error::last_os_error());
            }
        }
        Some(NtHeaders::Headers32(_)) => {
            let mut ctx = mem::zeroed::<WOW64_CONTEXT>();
            ctx.ContextFlags = WOW64_CONTEXT_ALL;
            if Wow64GetThreadContext(h_thread, &mut ctx) != 0 {
                ctx.Eip -= 1;
                dbg_cmd::x32::cmd_wait32(&mut ctx, h_proc, &mut h_thread, continue_dbg);
                if Wow64SetThreadContext(h_thread, &ctx) == 0 {
                    eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> error when setting thread context: {}", io::Error::last_os_error());
                }
            } else {
                eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> failed to get thread context: {}", io::Error::last_os_error());
            }
        }
        None => eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> how is it possible ? hmm suspicious that"),
    }
    if *continue_dbg {
        let mut b_insn = [0u8; 15];
        if ReadProcessMemory(h_proc, b_addr as LPVOID, b_insn.as_mut_ptr() as LPVOID, 15, &mut 0) == 0 {
            eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to get insn at address {:#x} : {}", b_addr, io::Error::last_os_error());
            return;
        }
        let mut decoder = Decoder::with_ip(NT_HEADER.unwrap().get_bitness() as u32, &b_insn, b_addr, DecoderOptions::NONE);
        let mut insn = Instruction::new();
        decoder.decode_out(&mut insn);
        let next_addr = b_addr + insn.len() as u64;
        let mut last_oc = 0u8;
        if ReadProcessMemory(h_proc, next_addr as LPVOID, ptr::addr_of_mut!(last_oc) as LPVOID, 1, &mut 0) == 0 {
            eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to read memory at address {:#x} : {}", next_addr, io::Error::last_os_error());
            return;
        }
        ALL_ELM.after_b.push(AfterB {
            last_addr_b: b_addr,
            after_b: next_addr,
            last_oc,
        });
        if WriteProcessMemory(h_proc, next_addr as LPVOID, &0xccu8 as *const u8 as LPVOID, 1, &mut 0) == 0 {
            eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to write memory at address {:#x} : {}", next_addr, io::Error::last_os_error());
            return;
        }
    }
    CloseHandle(h_thread);
}





pub unsafe fn set_breakpoint(h_proc: HANDLE, b_addr: u64) {
    let mut old_protect = 0;
    let mut written = 0;
    let mut original_byte: u8 = 0;
    if VirtualProtectEx(h_proc, b_addr as LPVOID, 1, PAGE_EXECUTE_READWRITE, &mut old_protect) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to change memory protection at address: 0x{:x} : {}", b_addr, io::Error::last_os_error());
        return;
    }
    if ReadProcessMemory(h_proc, b_addr as LPVOID, ptr::addr_of_mut!(original_byte) as LPVOID, 1, ptr::null_mut()) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to read memory at address: 0x{:x} : {}", b_addr, io::Error::last_os_error());
        return;
    }
    if original_byte == 0xcc {
        return;
    }
    SAVEINSN.push(SaveInsn { addr: b_addr, last_oc: original_byte });
    if WriteProcessMemory(h_proc, b_addr as LPVOID, &0xccu8 as *const u8 as LPVOID, 1, &mut written) == 0 || written != 1 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to write breakpoint at address: 0x{:x} : {}", b_addr, io::Error::last_os_error());
    }
    if VirtualProtectEx(h_proc, b_addr as LPVOID, 1, old_protect, &mut old_protect) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to restore memory protection at address: 0x{:x} : {}", b_addr, io::Error::last_os_error());
        return;
    }
    println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Breakpoint set at address: {:#x} in memory", b_addr);
}




pub unsafe fn set_breakpoint_in_ret_func(h_proc: HANDLE, debug_event: DEBUG_EVENT, addr: u64) {
    restore_byte_of_brkpt(h_proc, addr);
    let h_thread = OpenThread(THREAD_ALL_ACCESS, FALSE, debug_event.dwThreadId);
    if h_thread.is_null() {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to open thread : {}", io::Error::last_os_error());
        return;
    }
    ST_FRAME.clear();
    let rip = match NT_HEADER.unwrap() {
        NtHeaders::Headers32(_) => {
            let mut ctx = mem::zeroed::<WOW64_CONTEXT>();
            ctx.ContextFlags = WOW64_CONTEXT_ALL;
            if Wow64GetThreadContext(h_thread, &mut ctx) == 0 {
                eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to get thread context: {}", io::Error::last_os_error());
                return;
            }
            ctx.Eip -= 1;
            get_frame_st32(h_proc, h_thread, ctx);
            if Wow64SetThreadContext(h_thread, &ctx) == 0 {
                eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to sub 1 of rip : {}", io::Error::last_os_error());
                return;
            }
            ctx.Eip as u64
        }
        NtHeaders::Headers64(_) => {
            let mut ctx = mem::zeroed::<CONTEXT>();
            ctx.ContextFlags = CONTEXT_ALL;
            if GetThreadContext(h_thread, &mut ctx) == 0 {
                eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to get thread context: {}", io::Error::last_os_error());
                return;
            }
            ctx.Rip -= 1;
            get_frame_st(h_proc, h_thread, ctx);
            if SetThreadContext(h_thread, &ctx) == 0 {
                eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to sub 1 of rip : {}", io::Error::last_os_error());
                return;
            }
            ctx.Rip
        }
    };

    if let Some(frame) = get_real_frame(rip) {
        println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> address of return of function of insn {:#x} : {:#x}", addr - BASE_ADDR, frame.AddrReturn.Offset);
        ALL_ELM.break_va.push(frame.AddrReturn.Offset);
        set_breakpoint(h_proc, frame.AddrReturn.Offset);
    } else {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to get frame of the function of insn {:#x}", addr - BASE_ADDR);
    }
    SymCleanup(h_proc);
    CloseHandle(h_thread);
}
