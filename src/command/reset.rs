use crate::{Dbgoption, OPTION, pefile, ste, symbol, usage};
use crate::dbg::hook;
use crate::pefile::function;
use crate::symbol::Symbols;
use crate::log::*;

pub fn handle_reset(linev: &[&str]) {
    if linev.len() == 2 {
        let opt = linev[1].to_lowercase();
        let opt = opt.trim_start().trim_end();
        unsafe {
            match opt {
                "file" => {
                    OPTION.file = None;
                    println!("{VALID_COLOR}file context are reset{RESET_COLOR}");
                },
                "breakpoint" | "b" =>  {
                    OPTION.breakpoint_addr.clear();
                    println!("{VALID_COLOR}all breakpoint are clear{RESET_COLOR}");
                },
                "symbol" | "s" => {
                    *symbol::SYMBOLS_V = Symbols::default();
                    println!("{VALID_COLOR}all symbols have been unloaded{RESET_COLOR}");
                }
                "create-function" | "create-func" | "crt-func" => {
                    function::CR_FUNCTION.clear();
                    println!("{VALID_COLOR}all function created with cmd 'crt-func' has been deleted{RESET_COLOR}")
                }
                "hook" | "ho" => {
                    hook::HOOK_FUNC.clear();
                    println!("{VALID_COLOR}all hook set by cmd 'hook' has been deleted{RESET_COLOR}")
                }
                "stret" => {
                    ste::STE_RETURN_ADDR.clear();
                    println!("{VALID_COLOR}all the functions traced for their ret were clear{RESET_COLOR}");
                }
                "skip" => {
                    ste::ST_OVER_ADDR.clear();
                    println!("{VALID_COLOR}all functions specified with skip have been reset{RESET_COLOR}");
                }
                "args" | "arg" | "argv" => {
                    OPTION.arg = None;
                    println!("{VALID_COLOR}the arguments have been removed{RESET_COLOR}");
                }
                "all" => {
                    pefile::NT_HEADER = std::mem::zeroed();
                    ste::ST_OVER_ADDR.clear();
                    ste::STE_RETURN_ADDR.clear();
                    function::FUNC_INFO.clear();
                    function::CR_FUNCTION.clear();
                    hook::HOOK_FUNC.clear();
                    *symbol::SYMBOLS_V = Symbols::default();
                    symbol::IMAGE_BASE = 0;
                    *OPTION = Dbgoption::default();
                    println!("{VALID_COLOR}all element is cleared{RESET_COLOR}");
                },
                _ => eprintln!("{}", usage::USAGE_RESET)
            }
        }
    }
}