use std::io;
use std::io::Write;
use winapi::shared::ntdef::HANDLE;
use winapi::um::dbghelp::SymCleanup;
use winapi::um::processthreadsapi;
use winapi::um::winnt::CONTEXT;
use crate::{command, usage};
use crate::command::skip::SKIP_ADDR;
use crate::dbg::{BASE_ADDR, memory, RealAddr};
use crate::dbg::memory::stack::ST_FRAME;
use crate::utils::*;
use crate::pefile::function;
use crate::pefile::function::FUNC_INFO;
use crate::symbol::{SYMBOLS_V, SymbolType};

pub mod usages;
pub mod x32;
mod disasm;
pub mod x64;







fn init_cm(ctx: CONTEXT, process_handle: HANDLE, h_thread: HANDLE, addr_func: &mut u64) {
    unsafe {
        memory::stack::LEN = 0;
        ST_FRAME.clear();
        memory::stack::get_frame_st(process_handle, h_thread, ctx);
        *addr_func = if let Some(func) = FUNC_INFO.iter().find(|f|f.BeginAddress as u64 + BASE_ADDR <= ctx.Rip && f.EndAddress as u64 + BASE_ADDR >= ctx.Rip) {
            func.BeginAddress as u64 + BASE_ADDR
        }else {
            ctx.Rip
        };
        if SYMBOLS_V.symbol_type == SymbolType::PDB {
            memory::stack::get_local_sym(process_handle, *addr_func);
        }else {
            SymCleanup(process_handle);
        }
    }
}




fn unint_cm() {
    unsafe {
        for _ in 0..memory::stack::LEN {
            SYMBOLS_V.symbol_file.pop();
        }
    }
}






fn handle_backtrace(linev: &[&str]) {
    let count;
    let arg1 = linev.get(1);
    if arg1 == Some(&"full") || arg1.is_none() {
        count = usize::MAX;
    } else {
        match str_to::<usize>(arg1.unwrap()) {
            Ok(counts) => count = counts,
            Err(e) =>  {
                eprintln!("{ERR_COLOR}invalid count: {e}{RESET_COLOR}");
                return;
            }
        }
    }
    command::viewing::print_frame(count);
}





fn print_curr_func(addr_func: u64, ctx: CONTEXT) {
    unsafe {
        println!("{}Function    : {:#x} {}{RESET_COLOR}", ADDR_COLOR, addr_func, if let Some(sym) = SYMBOLS_V.symbol_file.iter().find(|s|s.real_addr64(ctx) == addr_func) {
            format!("<{}>", sym.name)
        }else {
            "".to_string()
        });
        if let Some(func) = FUNC_INFO.iter().find(|f|f.BeginAddress as u64 + BASE_ADDR == addr_func) {
            println!("{}End Address : {:#x}", VALUE_COLOR, func.EndAddress as u64 + BASE_ADDR);
            println!("{}Size        : {:#x}{RESET_COLOR}", MAGENTA, func.EndAddress - func.BeginAddress);
        }
    }
}





fn handle_crt_func(linev: &[&str], process_handle: HANDLE) {
    if linev.len() != 3 {
        println!("{}", usage::USAGE_CRT_FUNCTION);
        return;
    }
    let name = linev[1].to_string();
    let ret_value = match str_to::<u64>(linev[2]) {
        Ok(value) => value,
        Err(e) => {
            eprintln!("{ERR_COLOR}{e}{RESET_COLOR}");
            return;
        }
    };
    let mut crt_func = function::CrtFunc {
        name,
        ret_value,
        address: 0
    };
    unsafe { memory::set_cr_function(process_handle, &mut crt_func) }
}




fn handle_quit(input: &mut String, process_handle: HANDLE, continue_debugging: &mut bool, stop_process: &mut bool) {
    input.clear();
    print!("Are you sure to stop this process? [y/n] : ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(input).unwrap();
    if input.trim() == "y" {
        unsafe { processthreadsapi::TerminateProcess(process_handle, 0) };
        *continue_debugging = false;
        *stop_process = true;
    }
}



fn handle_ret(ctx: &mut CONTEXT) {
    unsafe {
        if let Some(frame_ret) = memory::stack::get_real_frame(ctx.Rip) {
            ctx.Rip = frame_ret.AddrReturn.Offset;
            ctx.Rsp -= 8;
            println!("{VALID_COLOR}now rip points to the address : {VALUE_COLOR}{:#x}{RESET_COLOR}\n{VALID_COLOR}and rsp was decremented by 8 : {VALUE_COLOR}{:#x}{RESET_COLOR}", ctx.Rip, ctx.Rsp);
        }
        else {
            eprintln!("{ERR_COLOR}a error has occured for get return addresse of the current stack frame : rip : {:#x}{RESET_COLOR}", ctx.Rip);
        }

    }
}



fn handle_skip(linev: &[&str], process_handle: HANDLE) {
    if linev.len() == 2 {
        let target = linev[1];
        let addr = match get_addr_br(target) {
            Ok(value) => value,
            Err(e) => {
                eprintln!("{e}");
                return;
            }
        };
        if addr == 0 {
            eprintln!("{ERR_COLOR}unknown symbol : {target}");
        } else {
            unsafe {
                SKIP_ADDR.push(addr);
                memory::set_addr_over(process_handle, addr);
            }
        }
    } else {
        println!("{}", usage::USAGE_SKIP);
    }
}





