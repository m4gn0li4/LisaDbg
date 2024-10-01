use crate::command::viewing;
use crate::dbg::dbg_cmd::usages;
use crate::dbg::memory::stack::LEN;
use crate::dbg::RealAddr;
use crate::symbol::SYMBOLS_V;
use crate::usage::USAGE_SYM_INFO;
use crate::utils::*;
use winapi::um::winnt::CONTEXT;

pub fn handle_sym_addr(linev: &[&str], ctx: CONTEXT) {
    if linev.len() != 2 {
        println!("{}", usages::USAGE_SA);
        return;
    }
    let name = linev[1];
    unsafe {
        if let Some(sym) = SYMBOLS_V.symbol_file.iter().find(|s| s.name == name) {
            println!(
                "{VALID_COLOR}the address of {name} is {:#x}",
                sym.real_addr64(ctx)
            );
        } else {
            eprintln!("{ERR_COLOR}the symbol {name} is unknow{RESET_COLOR}");
        }
    }
}

pub fn handle_sym_info(linev: &[&str], ctx: CONTEXT) {
    if linev.len() == 1 {
        println!("{USAGE_SYM_INFO}");
        return;
    }

    let sym_name = linev[1];
    if let Some(sym) = unsafe { SYMBOLS_V.symbol_file.iter().find(|s| s.name == sym_name) } {
        println!(
            "    {}name    : {}\
            \n    {}Address : {:#x}\
            \n    {}Type    : {}\
            \n    {}Size    : {:#x}\
            \n    {}File    : {}:{}\
            \n    {}Register: {} {}
            {RESET_COLOR}\n",
            DBG_COLOR, sym.name,
            ADDR_COLOR, sym.real_addr64(ctx),
            BLUE_COLOR, sym.types_e,
            MAGENTA, sym.size,
            WAR_COLOR, sym.filename, sym.line,
            VALID_COLOR, sym.register, viewing::frmrs(sym.register)
        );
    } else {
        eprintln!("{ERR_COLOR}The name of the symbol is unknown{RESET_COLOR}");
    }
}

pub fn print_local_sym(ctx: CONTEXT) {
    unsafe {
        let temp_sym = SYMBOLS_V.symbol_file.clone();
        SYMBOLS_V.symbol_file.reverse();
        SYMBOLS_V.symbol_file.truncate(LEN);
        viewing::print_sym(ctx);
        SYMBOLS_V.symbol_file = temp_sym;
    }
}
