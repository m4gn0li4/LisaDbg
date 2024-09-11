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

pub fn cmd_wait(ctx: &mut CONTEXT, process_handle: HANDLE, h_thread: HANDLE, continue_dbg: &mut bool) {
    let mut input = String::new();
    let mut stop_process = false;
    let mut addr_func = 0;
    dbg::dbg_cmd::init_cm(*ctx, process_handle, h_thread, &mut addr_func);

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
            Some(&"deref") => deref_mem::handle_deref(&linev, *ctx, process_handle),
            Some(&"set") => command::set::set_element64(process_handle, ctx, &linev),
            Some(&"q") | Some(&"quit") | Some(&"break") | Some(&"exit") => dbg::dbg_cmd::handle_quit(&mut input, continue_dbg, &mut stop_process),
            Some(&"base-addr") | Some(&"ba") => println!("base address : {VALUE_COLOR}{:#x}{RESET_COLOR}", unsafe { BASE_ADDR }),
            Some(&"b") | Some(&"breakpoint") => command::breakpoint::handle_breakpoint_proc(&linev, process_handle, *ctx),
            Some(&"reset") => command::reset::handle_reset(&linev),
            Some(&"remove") => command::remover::remove_element_proc(&linev, process_handle, ctx),
            Some(&"cva") => command::with_va::handle_calcule_va(&linev),
            Some(&"ret") => dbg::dbg_cmd::handle_ret(ctx),
            Some(&"break-ret") | Some(&"b-ret") => command::stret::handle_stret(&linev, process_handle),
            Some(&"skip") => dbg::dbg_cmd::handle_skip(&linev, process_handle),
            Some(&"view") => command::viewing::view_brpkt(&linev, *ctx, process_handle),
            Some(&"disasm") => disasm::handle_disasm(&linev, process_handle, *ctx),
            Some(&"crt-func") => dbg::dbg_cmd::handle_crt_func(&linev, process_handle),
            Some(&"s") => symbol::load_symbol(),
            Some(&"symbol-address")
            | Some(&"sym-address")
            | Some(&"sym-addr") => sym::handle_sym_addr(&linev, *ctx),
            Some(&"backtrace") | Some(&"frame") => dbg::dbg_cmd::handle_backtrace(&linev),
            Some(&"clear") => command::clear_cmd::clear_cmd(),
            Some(&"sym-info") => sym::handle_sym_info(&linev, *ctx),
            Some(&"add") => command::little_secret::add_op(&linev),
            Some(&"sub") => command::little_secret::sub_op(&linev),
            Some(&"watchpoint") | Some(&"watch") | Some(&"w") => command::watchpoint::watchpoint_proc(&linev, ctx),
            Some(&"crva") => command::with_va::handle_calcule_rva(&linev),
            Some(&"address-function") | Some(&"address-func") | Some(&"addr-func") => dbg::dbg_cmd::print_curr_func(addr_func, *ctx),
            Some(&"symbol-local") | Some(&"sym-local") => sym::print_local_sym(*ctx),
            Some(&"memory-info") | Some(&"mem-info") => memory::mem_info::handle_mem_info(&linev, process_handle, *ctx),
            Some(&"dettach") => {},
            Some(&"load") => command::load::load(&linev),
            Some(&"help") => usages::help(&linev),
            None => eprintln!("{ERR_COLOR}Please enter a command{RESET_COLOR}"),
            _ => eprintln!("{ERR_COLOR}Unknown command: {}{RESET_COLOR}", cmd.unwrap()),
        }
    }
    dbg::dbg_cmd::unint_cm();
}


