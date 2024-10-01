use crate::pefile::function::FUNC_INFO;
use crate::symbol::SYMBOLS_V;
use crate::utils::*;
use winapi::um::winnt::RUNTIME_FUNCTION;

fn get_(linev: &[&str]) -> usize {
    if linev.len() > 1 {
        linev.len() - 2
    } else {
        0
    }
}

pub fn get_address(linev: &[&str]) -> Result<u64, String> {
    let flag = linev[get_(linev)];
    let target = *linev.last().unwrap();
    if flag == "-a" || flag == "--address" {
        str_to::<u64>(target).map_err(|e| e.to_string())
    } else {
        unsafe {
            if let Some(sym) = SYMBOLS_V.symbol_file.iter().find(|s| s.name == target) {
                if sym.offset > 0 {
                    Ok(sym.offset as u64)
                } else {
                    Err("the specified symbol cannot have a negative offset".to_string())
                }
            } else {
                Err(format!("unknown symbol: '{}'", target))
            }
        }
    }
}

pub fn find_func_by_addr(addr: u64) -> Option<&'static RUNTIME_FUNCTION> {
    unsafe {
        FUNC_INFO.iter().find(|func| func.BeginAddress == addr as u32)
    }
}
