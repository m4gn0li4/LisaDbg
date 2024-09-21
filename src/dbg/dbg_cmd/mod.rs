use std::io;
use std::io::Write;
use winapi::shared::ntdef::HANDLE;
use winapi::um::dbghelp::SymCleanup;
use winapi::um::winbase::DebugSetProcessKillOnExit;
use winapi::um::winnt::CONTEXT;
use crate::{command, usage, ALL_ELM};
use crate::dbg::{BASE_ADDR, memory, RealAddr};
use crate::dbg::memory::stack::ST_FRAME;
use crate::utils::*;
use crate::pefile::NT_HEADER;
use crate::pefile::function::FUNC_INFO;
use crate::symbol::{SYMBOLS_V, SymbolType};

pub mod usages;
pub mod x32;
mod disasm;
pub mod x64;







fn init_cm(ctx: CONTEXT, h_proc: HANDLE, h_thread: HANDLE, addr_func: &mut u64) {
    unsafe {
        memory::stack::LEN = 0;
        ST_FRAME.clear();
        memory::stack::get_frame_st(h_proc, h_thread, ctx);
        *addr_func = if let Some(func) = FUNC_INFO.iter().find(|f|f.BeginAddress as u64 + BASE_ADDR <= ctx.Rip && f.EndAddress as u64 + BASE_ADDR >= ctx.Rip) {
            func.BeginAddress as u64 + BASE_ADDR
        }else {
            ctx.Rip
        };
        if SYMBOLS_V.symbol_type == SymbolType::PDB {
            memory::stack::get_local_sym(h_proc, *addr_func);
        }else {
            SymCleanup(h_proc);
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







fn handle_quit(input: &mut String, continue_debugging: &mut bool, stop_process: &mut bool) {
    input.clear();
    print!("Are you sure to stop this process? [y/n] : ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(input).unwrap();
    if input.trim() == "y" || input.trim() == "yes" {
        if unsafe {ALL_ELM.attach.is_some()} {
            loop {
                print!("kill attach process ? [y/n] : ");
                input.clear();
                io::stdout().flush().unwrap();
                io::stdin().read_line(input).unwrap();
                let input = input.trim();
                if input == "n" || input == "nop" {
                    unsafe { DebugSetProcessKillOnExit(0); }
                    break
                }
                else if input == "y" || input == "yes" {
                    unsafe { DebugSetProcessKillOnExit(1); }
                    break
                }
                else {
                    eprintln!("{ERR_COLOR}please choose a valid choice{RESET_COLOR}");
                    continue
                }
            }
        }
        *continue_debugging = false;
        *stop_process = true;
    }
}




fn handle_ret<T: NumConvert + num::Num + std::ops::SubAssign + std::fmt::LowerHex + Copy>(rip: &mut T, rsp: &mut T) {
    unsafe {
        if let Some(frame_ret) = memory::stack::get_real_frame(rip.to_u64()) {
            *rip = T::from_u64(frame_ret.AddrReturn.Offset);
            *rsp -= T::from_u64(NT_HEADER.unwrap().get_size_of_arch() as u64);
            println!(
                "{VALID_COLOR}now rip points to the address : {VALUE_COLOR}{:#x}{RESET_COLOR}\n\
                {VALID_COLOR}and rsp was decremented by {} : {VALUE_COLOR}{:#x}{RESET_COLOR}", *rip, NT_HEADER.unwrap().get_size_of_arch(), *rsp);
        } else {
            eprintln!("{ERR_COLOR}an error occurred while getting return address of the current stack frame: rip: {:#x}{RESET_COLOR}", *rip);
        }
    }
}



fn handle_skip(linev: &[&str], h_proc: HANDLE) {
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
                ALL_ELM.skip_addr.push(addr);
                memory::set_addr_over(h_proc, addr);
            }
        }
    } else {
        println!("{}", usage::USAGE_SKIP);
    }
}





