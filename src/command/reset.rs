use crate::cli::All;
use crate::{ALL_ELM, pefile, symbol, usage};
use crate::pefile::function;
use crate::symbol::{Symbols, SYMBOLS_V};
use crate::utils::*;

pub fn handle_reset(linev: &[&str]) {
    if linev.len() == 2 {
        let opt = linev[1].to_lowercase();
        let opt = opt.trim_start().trim_end();
        unsafe {
            match opt {
                "file" => {
                    *SYMBOLS_V = Symbols::default();
                    ALL_ELM.file = None;
                    println!("{VALID_COLOR}file context are reset{RESET_COLOR}");
                },
                "breakpoint" | "b" =>  {
                    ALL_ELM.break_rva.clear();
                    println!("{VALID_COLOR}all breakpoint are clear{RESET_COLOR}");
                },
                "symbol" | "s" => {
                    *SYMBOLS_V = Symbols::default();
                    println!("{VALID_COLOR}all symbols have been unloaded{RESET_COLOR}");
                }
                "hook" | "ho" => {
                    ALL_ELM.hook.clear();
                    println!("{VALID_COLOR}all hook set by cmd 'hook' has been deleted{RESET_COLOR}")
                }
                "break-ret" | "b-ret" => {
                    ALL_ELM.break_ret.clear();
                    println!("{VALID_COLOR}all the functions traced for their ret were clear{RESET_COLOR}");
                }
                "skip" => {
                    ALL_ELM.skip_addr.clear();
                    println!("{VALID_COLOR}all functions specified with skip have been reset{RESET_COLOR}");
                }
                "args" | "arg" | "argv" => {
                    ALL_ELM.arg = None;
                    println!("{VALID_COLOR}the arguments have been removed{RESET_COLOR}");
                }
                "watchpoint" | "watchpts" | "w" => {
                    ALL_ELM.watchpts.clear();
                    println!("{VALID_COLOR}all watchpoint has been removed{RESET_COLOR}");
                }
                "all" => {
                    pefile::NT_HEADER = std::mem::zeroed();
                    ALL_ELM.skip_addr.clear();
                    ALL_ELM.break_ret.clear();
                    function::FUNC_INFO.clear();
                    ALL_ELM.hook.clear();
                    *SYMBOLS_V = Symbols::default();
                    symbol::IMAGE_BASE = 0;
                    *ALL_ELM = All::default();
                    println!("{VALID_COLOR}all element is cleared{RESET_COLOR}");
                },
                _ => eprintln!("{}", usage::USAGE_RESET)
            }
        }
    }
}