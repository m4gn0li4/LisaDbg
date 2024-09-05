use std::ptr::addr_of;
use winapi::um::winnt::{CONTEXT, RUNTIME_FUNCTION};
use crate::{OPTION, symbol, usage};
use crate::command::skip::SKIP_ADDR;
use crate::command::stret::BREAK_RET;
use crate::dbg::{BASE_ADDR, memory, RealAddr};
use crate::utils::*;
use crate::pefile::function::{CR_FUNCTION, FUNC_INFO};
use crate::pefile::section::SECTION_VS;
use crate::symbol::SYMBOLS_V;

pub fn view_brpkt(linev: &[&str], ctx: CONTEXT) {
    if linev.len() != 2 {
        println!("{}", usage::USAGE_VIEW);
        return;
    }
    let elm = linev[1];
    match elm {
        "breakpoint" | "brpt" | "b" => print_elements(unsafe { &*addr_of!(OPTION.breakpoint_addr) }),
        "skip" => print_elements(unsafe { &*addr_of!(SKIP_ADDR) }),
        "stret" => print_elements(unsafe { &*addr_of!(BREAK_RET) }),
        "symbol" | "sym" | "s" => print_sym(ctx),
        "hook-func" | "hook" | "h" => print_hook_func(),
        "create-function" | "create-func" | "crt-func" => print_crt_func(),
        "watchpoint" | "watch" | "w" => print_watchpt(ctx),
        "function" | "func" | "f" => print_function(),
        "section" | "sec" => print_section(),
        "hmodule" | "module" => {},
        _ => eprintln!("{ERR_COLOR}unknow option : '{elm}'{RESET_COLOR}"),
    }
}







fn print_section() {
    unsafe {
        for (i, section) in SECTION_VS.iter().enumerate() {
            println!("\n{VALID_COLOR}#{i}: \
            \n     {}Name         : {}\
            \n     {}Address      : {:#x}\
            \n     {}Size of code : {:#x}{RESET_COLOR}",
            DBG_COLOR, section.name,
            ADDR_COLOR, section.addr as u64 + BASE_ADDR,
            VALUE_COLOR, section.content.len())
        }
    }
}



fn print_function() {
    unsafe {
        for (i, func) in (&*addr_of!(FUNC_INFO)).iter().enumerate() {
            println!("\n{VALID_COLOR}func_#{i}:\
        \n     {}Address     : {:#x} {}\
        \n     {}end-address : {:#x}\
        \n     {}size        : {:#x}{RESET_COLOR}",
                ADDR_COLOR, func.BeginAddress as u64 + BASE_ADDR, get_sym_name(func),
                VALUE_COLOR, func.EndAddress as u64 + BASE_ADDR,
                MAGENTA, func.EndAddress - func.BeginAddress,
            )
        }
    }
}


fn get_sym_name(func: &RUNTIME_FUNCTION) -> String {
    return if let Some(sym) = unsafe {SYMBOLS_V.symbol_file.iter().find(|s|s.offset + BASE_ADDR as i64 == func.BeginAddress as i64 + BASE_ADDR as i64)} {
        format!("<{}>", sym.name)
    }else {
        "".to_string()
    }
}


fn print_elements<T: IntoIterator>(elements: T) where T::Item: std::fmt::LowerHex {
    for (i, e) in elements.into_iter().enumerate() {
        println!("{i} : {VALUE_COLOR}{:#x}{RESET_COLOR}", e);
    }
}



fn print_watchpt(ctx: CONTEXT) {
    for (i, watchpts) in unsafe {OPTION.watchpts.iter().enumerate()} {
        println!("{DBG_COLOR}{i}: \
        \n     {}memory zone    : {}\
        \n     {}check access   : {:?}\
        \n     {}offset         : {}\
        \n     {}size           : {:#x}{RESET_COLOR}",
                 CYAN_COLOR, watchpts.flag_type_mem,
                 BYTES_COLOR, watchpts.check_type,
                 ADDR_COLOR, watchpts.format_offset(ctx),
                 VALID_COLOR, watchpts.memory_size
        );
    }
}



pub fn print_hook_func() {
    for (i, hook) in unsafe { crate::command::hook::HOOK_FUNC.iter().enumerate() } {
        println!("{VALUE_COLOR}{i}{RESET_COLOR}:\
            \n     {WAR_COLOR}Target   : {DBG_COLOR}{:#x}\
            \n     {WAR_COLOR}Replace  : {MAGENTA}{:#x}{RESET_COLOR}\n",
            hook.target, hook.replacen
        );
    }
}



fn print_crt_func() {
    for (i, sym) in unsafe { CR_FUNCTION.iter().enumerate() } {
        println!("{VALUE_COLOR}{i}{RESET_COLOR}: \
        \n     {}Name     : {}\
        \n     {}Address  : {}\
        \n     {}ret-value: {:#x}{RESET_COLOR}\n",
                 DBG_COLOR, sym.name,
                 ADDR_COLOR, if sym.address == 0 {"unitialized".to_string()} else {format!("{:#x}", sym.address)},
                 VALUE_COLOR, sym.ret_value
        );
    }
}


pub fn print_sym(ctx: CONTEXT) {
    println!("{VALID_COLOR}Symbol type: {VALUE_COLOR}{}{RESET_COLOR}", unsafe { SYMBOLS_V.symbol_type });
    for (i, sym) in unsafe { SYMBOLS_V.symbol_file.iter().enumerate() } {
        println!(
            "{CYAN_COLOR}{i}{RESET_COLOR}:\
            \n     {}Name     : {}\
            \n     {}\
            \n     {}Type     : {}\
            \n     {}Size     : {:#x}\
            \n     {}file     : {}:{}\
            \n     {}register : {} {}
            {RESET_COLOR}\n",
            DBG_COLOR, sym.name,
            if unsafe{BASE_ADDR == 0}  {
                format!("{}offset   : {:#x}", ADDR_COLOR, sym.offset)
            }else {
                format!("{}address  : {:#x} (offset={})", ADDR_COLOR, sym.real_addr64(ctx), sym.offset)
            },
            BLUE_COLOR, sym.types_e,
            MAGENTA, sym.size,
            WAR_COLOR, sym.filename, sym.line,
            VALID_COLOR, sym.register, frmrs(sym.register)
        );
    }
}



pub fn frmrs(reg_field: u32) -> String{
    let s_reg = symbol::pdb::get_reg_with_reg_field(reg_field);
    if s_reg != "" {
        format!("({})", s_reg.to_uppercase())
    }else {
        "".to_string()
    }
}

pub fn print_frame(count: usize) {
    unsafe {
        for i in 0..count {
            if let Some(frame) = memory::stack::ST_FRAME.get(i) {
                println!("\n{}Frame #{}:", VALID_COLOR, i);
                let get_function_and_symbol = |offset: u64| {
                    FUNC_INFO.iter().find(|f| f.BeginAddress as u64 + BASE_ADDR <= offset && f.EndAddress as u64 + BASE_ADDR >= offset)
                        .map(|func| {
                            if let Some(sym) = SYMBOLS_V.symbol_file.iter().find(|s| s.offset == func.BeginAddress as i64) {
                                format!("<{}{:+}>", sym.name, offset as i64 - (func.BeginAddress as i64 + BASE_ADDR as i64))
                            } else {
                                format!("<func_{:x}{:+}", func.BeginAddress, offset as i64 - (func.BeginAddress as i64 + BASE_ADDR as i64))
                            }
                        })
                        .unwrap_or_else(|| "".to_string())
                };
                println!("{}   Insn ptr (rip)      : {}{:#18x} {}", ADDR_COLOR, VALUE_COLOR, frame.AddrPC.Offset, get_function_and_symbol(frame.AddrPC.Offset));
                println!("{}   Return Address      : {}{:#18x} {}", ADDR_COLOR, BYTES_COLOR, frame.AddrReturn.Offset, get_function_and_symbol(frame.AddrReturn.Offset));
                println!("{}   Frame Ptr           : {}{:#18x}", ADDR_COLOR, SYM_COLOR, frame.AddrFrame.Offset);
                println!("{}   Stack Ptr (rsp)     : {}{:#18x}", ADDR_COLOR, DBG_COLOR, frame.AddrStack.Offset);
            } else {
                if count != usize::MAX {
                    println!("{WAR_COLOR}the count is greater than the total number of frames, frame: {} count: {}", memory::stack::ST_FRAME.len(), count);
                }
                return;
            }
        }
    }
}

