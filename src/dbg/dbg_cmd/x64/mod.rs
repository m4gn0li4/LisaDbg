use std::io::{self, Write};
use std::str;
use winapi::shared::ntdef::HANDLE;
use winapi::um::winnt::CONTEXT;
use crate::{command, dbg, symbol};
use crate::command::sym;
use crate::dbg::{memory, BASE_ADDR};
use crate::dbg::dbg_cmd::{disasm, usages};
use crate::dbg::memory::deref_mem;
use crate::utils::*;

pub mod modifier;
pub mod info_reg;

pub fn cmd_wait(ctx: &mut CONTEXT, h_proc: HANDLE, h_thread: HANDLE, continue_dbg: &mut bool) {
    let mut input = String::new();
    let mut stop_intp = false;
    let mut addr_func = 0;
    dbg::dbg_cmd::init_cm(*ctx, h_proc, h_thread, &mut addr_func);

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
            Some(&"v") | Some(&"value") => unsafe { info_reg::handle_reg(&linev, *ctx) },
            Some(&"deref") => deref_mem::handle_deref(&linev, *ctx, h_proc),
            Some(&"set") => command::set::set_element64(h_proc, ctx, &linev),
            Some(&"q") | Some(&"quit") | Some(&"break") | Some(&"exit") => dbg::dbg_cmd::handle_quit(&mut input, continue_dbg, &mut stop_intp),
            Some(&"base-addr") | Some(&"ba") => println!("base address : {VALUE_COLOR}{:#x}{RESET_COLOR}", unsafe { BASE_ADDR }),
            Some(&"b") | Some(&"breakpoint") => command::breakpoint::handle_breakpoint_proc(&linev, h_proc, *ctx),
            Some(&"reset") => command::reset::handle_reset(&linev),
            Some(&"remove") => command::remover::remove_element_proc(&linev, h_proc, ctx),
            Some(&"cva") => command::with_va::handle_calcule_va(&linev),
            Some(&"ret") => dbg::dbg_cmd::handle_ret::<u64>(&mut ctx.Rip, &mut ctx.Rsp),
            Some(&"break-ret") | Some(&"b-ret") => command::stret::handle_stret(&linev, h_proc),
            Some(&"skip") => dbg::dbg_cmd::handle_skip(&linev, h_proc),
            Some(&"view") => command::viewing::view_brpkt(&linev, *ctx, h_proc),
            Some(&"disasm") => disasm::handle_disasm(&linev, h_proc, *ctx),
            Some(&"s") => symbol::load_symbol(),
            Some(&"symbol-address")
            | Some(&"sym-address")
            | Some(&"sym-addr") => sym::handle_sym_addr(&linev, *ctx),
            Some(&"backtrace") | Some(&"frame") => dbg::dbg_cmd::handle_backtrace(&linev),
            Some(&"sym-info") => sym::handle_sym_info(&linev, *ctx),
            Some(&"add") => command::little_secret::add_op(&linev),
            Some(&"sub") => command::little_secret::sub_op(&linev),
            Some(&"watchpoint") | Some(&"watch") | Some(&"w") => command::watchpoint::watchpoint_proc(&linev, ctx),
            Some(&"crva") => command::with_va::handle_calcule_rva(&linev),
            Some(&"address-function") | Some(&"address-func") | Some(&"addr-func") => dbg::dbg_cmd::print_curr_func(addr_func, *ctx),
            Some(&"symbol-local") | Some(&"sym-local") => sym::print_local_sym(*ctx),
            Some(&"memory-info") | Some(&"mem-info") => memory::mem_info::handle_mem_info64(&linev, h_proc, *ctx),
            Some(&"b-va") | Some(&"break-va") => command::breakpoint::handle_b_va_proc(&linev, h_proc),
            Some(&"def") => command::def::handle_def(&linev),
            Some(&"help") => usages::help(&linev),
            Some(&"clear") | Some(&"cls") => command::clear_cmd::clear_cmd(),
            None => eprintln!("{ERR_COLOR}Please enter a command{RESET_COLOR}"),
            _ => eprintln!("{ERR_COLOR}Unknown command: {}{RESET_COLOR}", cmd.unwrap()),
        }
    }
    dbg::dbg_cmd::unint_cm();
}


