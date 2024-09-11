use std::str::FromStr;
use std::num::ParseIntError;
use anyhow::Error;
use num::Num;
use winapi::um::winnt::{CONTEXT, WOW64_CONTEXT};
use crate::dbg::dbg_cmd::x32::info_reg::ToValue32;
use crate::dbg::dbg_cmd::x64::info_reg::{ToValue, Value};
use crate::dbg::RealAddr;
use crate::symbol::SYMBOLS_V;

pub const ERR_COLOR: &str = "\x1b[31m";
pub const DBG_COLOR: &str = "\x1b[92m";
pub const INSTR_COLOR: &str = "\x1b[92m";

pub const RESET_COLOR: &str = "\x1b[0m";
pub const VALUE_COLOR: &str = "\x1b[96m";
pub const VALID_COLOR: &str = "\x1b[32m";
pub const WAR_COLOR: &str = "\x1B[93m";
pub const BYTES_COLOR: &str = "\x1b[93m";
pub const ADDR_COLOR: &str = "\x1b[95m";
pub const SYM_COLOR: &str = "\x1b[94m";
pub const CYAN_COLOR: &str = "\x1b[36m";
pub const MAGENTA: &str = "\x1b[35m";
pub const BLUE_COLOR: &str = "\x1b[34m";




pub fn str_to<T: Num<FromStrRadixErr = ParseIntError> + FromStr<Err = ParseIntError>>(addr_str: &str) -> Result<T, Error> {
    if addr_str.to_lowercase().contains("0x") {
        Ok(T::from_str_radix(&addr_str.replace("0x", ""), 16)?)
    }
    else if addr_str.to_lowercase().ends_with("h") {
        Ok(T::from_str_radix(&addr_str[..addr_str.len()-1], 16)?)
    }
    else {
        Ok(addr_str.parse()?)
    }
}




pub fn get_addr_va(addr_str: &str, ctx: CONTEXT) -> Result<u64, String> {
    match str_to::<u64>(addr_str){
        Ok(addr) => Ok(addr),
        Err(e) => unsafe {
            if e.to_string().contains("invalid digit") {
                if let Some(sym) = SYMBOLS_V.symbol_file.iter().find(|s|s.name == addr_str) {
                    Ok(sym.real_addr64(ctx))
                }else {
                    match ctx.str_to_value_ctx(addr_str) {
                        Value::U64(addr) => Ok(addr),
                        _ => {
                            Err(format!("{}Invalid target: '{addr_str}'{}", ERR_COLOR, RESET_COLOR))
                        }
                    }
                }
            }else {
                Err(format!("{}Invalid target: '{addr_str}'{}", ERR_COLOR, RESET_COLOR))
            }
        }
    }
}





pub fn get_addr_va32(addr_str: &str, ctx: WOW64_CONTEXT) -> Result<u32, String> {
    match str_to::<u32>(addr_str){
        Ok(addr) => Ok(addr),
        Err(e) => unsafe {
            if e.to_string().contains("invalid digit") {
                if let Some(sym) = SYMBOLS_V.symbol_file.iter().find(|s|s.name == addr_str) {
                    Ok(sym.real_addr32(ctx))
                }else {
                    let ad = ctx.str_to_ctx(addr_str);
                    if ad != 0 {
                        Ok(ad)
                    }else {
                        Err(format!("{}Invalid target: '{addr_str}'{}", ERR_COLOR, RESET_COLOR))
                    }
                }
            }else {
                Err(format!("{}Invalid target: '{addr_str}'{}", ERR_COLOR, RESET_COLOR))
            }
        }
    }
}

pub fn get_addr_br(addr_str: &str) -> Result<u64, String> {
    match str_to::<u64>(addr_str) {
        Ok(value) => Ok(value),
        Err(_) => unsafe {
            if let Some(sym) = SYMBOLS_V.symbol_file.iter().find(|s|s.name == addr_str) {
                if sym.offset > 0 {
                    Ok(sym.offset as u64)
                }else {
                    return Err(format!("{ERR_COLOR}the specified symbol cannot have a negative offset{RESET_COLOR}"));
                }
            }else {
                return Err(format!("{ERR_COLOR}invalid target : {}{RESET_COLOR}", addr_str));
            }
        }
    }
}

