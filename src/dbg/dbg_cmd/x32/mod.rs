mod deref_mem32;
pub mod info_reg;
mod modifier32;
mod disasm;

use crate::utils::*;
use std::io;
use std::io::Write;
use winapi::shared::ntdef::HANDLE;
use winapi::um::dbghelp::SymCleanup;
use winapi::um::winnt::WOW64_CONTEXT;
use crate::{command, usage};
use crate::command::sym;
use crate::dbg::{BASE_ADDR, memory};
use crate::dbg::dbg_cmd::*;
use crate::pefile::function;
use crate::symbol::SymbolType;

fn init_cm(ctx: WOW64_CONTEXT, process_handle: HANDLE, h_thread: HANDLE, addr_func: &mut u32) {
    unsafe {
        ST_FRAME.clear();
        memory::stack::get_frame_st32(process_handle, h_thread, ctx);
        *addr_func = if let Some(func) = FUNC_INFO.iter().find(|f|f.BeginAddress + BASE_ADDR as u32 <= ctx.Eip && f.EndAddress + BASE_ADDR as u32 >= ctx.Eip) {
            func.BeginAddress + BASE_ADDR as u32
        }else {
            ctx.Eip
        };
        if SYMBOLS_V.symbol_type == SymbolType::PDB {
            memory::stack::get_local_sym(process_handle, *addr_func as u64);
        }else {
            SymCleanup(process_handle);
        }
    }
}





pub fn cmd_wait32(ctx: &mut WOW64_CONTEXT, process_handle: HANDLE, h_thread: HANDLE, continue_debugging: &mut bool) {
    let mut input = String::new();
    let mut stop_process = false;
    let mut addr_func = 0;
    init_cm(*ctx, process_handle, h_thread, &mut addr_func);

    while !stop_process {
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
            Some(&"v") | Some(&"value") => unsafe { info_reg::handle_reg(&linev, *ctx) },
            Some(&"deref") => deref_mem32::handle_deref32(&linev, *ctx, process_handle),
            Some(&"setr") | Some(&"setreg") => modifier32::register::handle_set_register(&linev, ctx),
            Some(&"q") | Some(&"quit") | Some(&"break") => handle_quit(&mut input, continue_debugging, &mut stop_process),
            Some(&"base-addr") | Some(&"ba") => println!("base address : {VALUE_COLOR}{:#x}{RESET_COLOR}", unsafe { BASE_ADDR }),
            Some(&"setm") | Some(&"setmemory") => modifier32::set_mem::handle_set_memory(process_handle, *ctx, &linev),
            Some(&"b") | Some(&"breakpoint") => command::breakpoint::handle_breakpoint_proc32(&linev, process_handle, *ctx),
            Some(&"rb") => command::breakpoint::handle_restore_breakpoint_proc(&linev, process_handle),
            Some(&"reset") => command::reset::handle_reset(&linev),
            Some(&"remove") => command::remover::remove_element(&linev),
            Some(&"cva") => command::with_va::handle_calcule_va(&linev),
            Some(&"ret") => handle_ret32(ctx),
            Some(&"break-ret") | Some(&"b-ret") => command::stret::handle_stret(&linev, process_handle),
            Some(&"skip") => handle_skip(&linev, process_handle),
            Some(&"view") => command::viewing::view_brpkt(&linev, wow64_context_to_context(*ctx)),
            Some(&"help") => usages::help(&linev),
            Some(&"backtrace") | Some(&"frame") => handle_backtrace(&linev, process_handle, h_thread, ctx),
            Some(&"disasm") => disasm::handle_disasm32(&linev, process_handle, *ctx),
            Some(&"crt-func") => handle_crt_func32(&linev, process_handle),
            Some(&"sym-info") => command::sym::handle_sym_info(&linev, wow64_context_to_context(*ctx)),
            Some(&"address-function") | Some(&"address-func") | Some(&"addr-func") => print_curr_func(addr_func as u64, wow64_context_to_context(*ctx)),
            Some(&"symbol-local") | Some(&"sym-local") => sym::print_local_sym(wow64_context_to_context(*ctx)),
            Some(&"memory-info") | Some(&"mem-info") => memory::mem_info::handle_mem_info(&linev, process_handle, wow64_context_to_context(*ctx)),
            Some(&"load") => command::load::load(&linev),
            Some(&"add") => command::little_secret::add_op(&linev),
            Some(&"sub") => command::little_secret::sub_op(&linev),
            None => eprintln!("{ERR_COLOR}Please enter a command{RESET_COLOR}"),
            _ => eprintln!("{ERR_COLOR}Unknown command: {}{RESET_COLOR}", cmd.unwrap()),
        }
    }
    unint_cm();
}


pub fn wow64_context_to_context(wow64_context: WOW64_CONTEXT) -> CONTEXT {
    let mut context: CONTEXT = unsafe { std::mem::zeroed() };
    context.Rax = wow64_context.Eax as u64;
    context.Rbx = wow64_context.Ebx as u64;
    context.Rcx = wow64_context.Ecx as u64;
    context.Rdx = wow64_context.Edx as u64;
    context.Rsi = wow64_context.Esi as u64;
    context.Rdi = wow64_context.Edi as u64;
    context.Rsp = wow64_context.Esp as u64;
    context.Rbp = wow64_context.Ebp as u64;
    context.Rip = wow64_context.Eip as u64;
    context
}




fn handle_backtrace(linev: &[&str], process_handle: HANDLE, h_thread: HANDLE, ctx: &mut WOW64_CONTEXT) {
    let count;
    if linev.len() == 1 || linev[1] == "full"{
        count = usize::MAX;
    }else {
        match str_to::<usize>(linev[1]) {
            Ok(counts) => count = counts,
            Err(e) => {
                eprintln!("{ERR_COLOR}invalid count : {e}{RESET_COLOR}");
                return;
            }
        }
    }
    unsafe { memory::stack::get_frame_st32(process_handle, h_thread, *ctx) }
    command::viewing::print_frame(count);
}




fn handle_ret32(ctx: &mut WOW64_CONTEXT) {
    unsafe {
        if let Some(frame) = memory::stack::get_real_frame(ctx.Eip as u64) {
            ctx.Eip = frame.AddrReturn.Offset as u32;
            ctx.Esp -= 4;
            println!("{VALID_COLOR}now rip points to the address : {VALUE_COLOR}{:#x}{RESET_COLOR}\n{VALID_COLOR}and esp was decremented by 4 : {VALUE_COLOR}{:#x}{RESET_COLOR}", ctx.Eip, ctx.Esp);
        }
        else {
            eprintln!("{ERR_COLOR}a error has occured for get return addresse of the current stack frame : eip : {:#x}{RESET_COLOR}", ctx.Eip);
        }
    }
}




fn handle_crt_func32(linev: &[&str], process_handle: HANDLE) {
    if linev.len() != 3 {
        println!("{}", usage::USAGE_CRT_FUNCTION);
        return;
    }
    let name = linev[1].to_string();
    let ret_value = match str_to::<u32>(linev[2]) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("{ERR_COLOR}{e}{RESET_COLOR}");
            return;
        }
    } as u64;
    let mut crt_func = function::CrtFunc {
        name,
        ret_value,
        address: 0
    };
    unsafe { memory::set_cr_function(process_handle, &mut crt_func) }
}




