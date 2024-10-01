mod deref_mem32;
pub mod info_reg;
pub mod modifier32;

use crate::command;
use crate::command::{hook, sym};
use crate::dbg::dbg_cmd::*;
use crate::dbg::memory::set::thread::get_thread_now;
use crate::dbg::{memory, BASE_ADDR};
use crate::symbol::SymbolType;
use std::io::Write;
use std::{io, ptr};
use winapi::shared::ntdef::HANDLE;
use winapi::um::dbghelp::SymCleanup;
use winapi::um::winnt::WOW64_CONTEXT;

pub fn init_cm(ctx: WOW64_CONTEXT, h_proc: HANDLE, h_thread: HANDLE, addr_func: &mut u32) {
    unsafe {
        ST_FRAME.clear();
        memory::stack::get_frame_st32(h_proc, h_thread, ctx);
        *addr_func = if let Some(func) = FUNC_INFO.iter().find(|f| { f.BeginAddress + BASE_ADDR as u32 <= ctx.Eip
                && f.EndAddress + BASE_ADDR as u32 >= ctx.Eip
        }) {
            func.BeginAddress + BASE_ADDR as u32
        } else {
            ctx.Eip
        };
        if SYMBOLS_V.symbol_type == SymbolType::PDB {
            memory::stack::get_local_sym(h_proc, *addr_func as u64);
        } else {
            SymCleanup(h_proc);
        }
    }
}

pub fn cmd_wait32(ctx: &mut WOW64_CONTEXT, h_proc: HANDLE, h_thread: &mut HANDLE, c_dbg: &mut bool) {
    let mut input = String::new();
    let mut stop_intp = false;
    let mut addr_func = 0;
    init_cm(*ctx, h_proc, *h_thread, &mut addr_func);

    while !stop_intp {
        input.clear();
        print!("\x1b[38;5;129m>> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        print!("{RESET_COLOR}");
        io::stdout().flush().unwrap();
        let linev: Vec<&str> = input.split_whitespace().collect();
        let cmd = linev.first();
        match cmd {
            Some(&"c") | Some(&"continue") | Some(&"run") => break,
            Some(&"v") | Some(&"value") | Some(&"registers") | Some(&"register") | Some(&"reg") | Some(&"r") => unsafe { info_reg::handle_reg(&linev, *ctx) },
            Some(&"deref") => deref_mem32::handle_deref32(&linev, *ctx, h_proc),
            Some(&"q") | Some(&"quit") | Some(&"break") => handle_quit(&mut input, c_dbg, &mut stop_intp),
            Some(&"base-addr") | Some(&"ba") => println!("base address : {VALUE_COLOR}{:#x}{RESET_COLOR}", unsafe { BASE_ADDR }),
            Some(&"set") => command::set::set_element32(h_proc, ctx, &linev),
            Some(&"b") | Some(&"breakpoint") => command::breakpoint::handle_breakpoint_proc32(&linev, h_proc, *ctx),
            Some(&"rb") => command::breakpoint::handle_restore_breakpoint_proc(&linev, h_proc),
            Some(&"reset") => command::reset::reset_proc(&linev, h_proc),
            Some(&"remove") => command::remover::remove_element(&linev),
            Some(&"cva") => command::with_va::handle_calcule_va(&linev),
            Some(&"ret") => handle_ret::<u32>(&mut ctx.Eip, &mut ctx.Esp),
            Some(&"break-ret") | Some(&"b-ret") => command::stret::handle_stret(&linev, h_proc),
            Some(&"skip") => handle_skip(&linev, h_proc),
            Some(&"view") => command::viewing::view_brpkt(&linev, wow64_context_to_context(*ctx), h_proc),
            Some(&"help") => usages::help(&linev),
            Some(&"backtrace") | Some(&"frame") => handle_backtrace(&linev),
            Some(&"disasm") => disasm::handle_disasm32(&linev, h_proc, *ctx),
            Some(&"sym-info") => sym::handle_sym_info(&linev, wow64_context_to_context(*ctx)),
            Some(&"address-function") | Some(&"address-func") | Some(&"addr-func") => print_curr_func(addr_func as u64, wow64_context_to_context(*ctx)),
            Some(&"symbol-local") | Some(&"sym-local") => sym::print_local_sym(wow64_context_to_context(*ctx)),
            Some(&"memory-info") | Some(&"mem-info") => memory::mem_info::handle_mem_info32(&linev, h_proc, *ctx),
            Some(&"def") => command::def::handle_def(&linev),
            Some(&"find") => memory::finder::handle_find(&linev, h_proc),
            Some(&"b-va") | Some(&"break-va") => command::breakpoint::handle_b_va_proc(&linev, h_proc),
            Some(&"add") => command::little_secret::add_op(&linev),
            Some(&"sub") => command::little_secret::sub_op(&linev),
            Some(&"debug-thread") | Some(&"dbg-thread") | Some(&"dbg-th") => {
                let mut addr = addr_func as u64;
                memory::set::thread::change_dbg_thread(&linev, ptr::addr_of_mut!(*ctx) as *mut CONTEXT, h_proc, h_thread, &mut addr);
                addr_func = addr as u32;
            }
            Some(&"thread-info") | Some(&"th-info") => get_thread_now(*h_thread),
            Some(&"hook") => hook::handle_hook_proc(&linev, h_proc),
            Some(&"b-ret-va") | Some(&"b-retva") => command::stret::handle_proc_b_ret_va(&linev, h_proc),
            Some(&"clear") | Some(&"cls") => command::clear_cmd::clear_cmd(),
            None => eprintln!("{ERR_COLOR}Please enter a command{RESET_COLOR}"),
            _ => eprintln!("{ERR_COLOR}Unknown command: {}{RESET_COLOR}", cmd.unwrap()),
        }
    }
    unint_cm();
}

pub fn wow64_context_to_context(wow64_ctx: WOW64_CONTEXT) -> CONTEXT {
    let mut ctx: CONTEXT = unsafe { std::mem::zeroed() };
    ctx.Rax = wow64_ctx.Eax as u64;
    ctx.Rbx = wow64_ctx.Ebx as u64;
    ctx.Rcx = wow64_ctx.Ecx as u64;
    ctx.Rdx = wow64_ctx.Edx as u64;
    ctx.Rsi = wow64_ctx.Esi as u64;
    ctx.Rdi = wow64_ctx.Edi as u64;
    ctx.Rsp = wow64_ctx.Esp as u64;
    ctx.Rbp = wow64_ctx.Ebp as u64;
    ctx.Rip = wow64_ctx.Eip as u64;
    ctx.Dr0 = wow64_ctx.Dr0 as u64;
    ctx.Dr1 = wow64_ctx.Dr1 as u64;
    ctx.Dr2 = wow64_ctx.Dr2 as u64;
    ctx.Dr3 = wow64_ctx.Dr3 as u64;
    ctx.Dr6 = wow64_ctx.Dr6 as u64;
    ctx.Dr7 = wow64_ctx.Dr7 as u64;
    ctx.ContextFlags = wow64_ctx.ContextFlags;
    ctx
}

fn handle_backtrace(linev: &[&str]) {
    let count;
    if linev.len() == 1 || linev[1] == "full" {
        count = usize::MAX;
    } else {
        match str_to::<usize>(linev[1]) {
            Ok(counts) => count = counts,
            Err(e) => {
                eprintln!("{ERR_COLOR}invalid count : {e}{RESET_COLOR}");
                return;
            }
        }
    }
    command::viewing::print_frame(count);
}
