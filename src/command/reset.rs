use winapi::um::winnt::HANDLE;
use crate::cli::All;
use crate::pefile::function;
use crate::symbol::{Symbols, SYMBOLS_V};
use crate::utils::*;
use crate::{pefile, symbol, usage, ALL_ELM};
use crate::dbg::{memory, BASE_ADDR};

fn clear_symbols() {
    unsafe {
        *SYMBOLS_V = Symbols::default();
        symbol::IMAGE_BASE = 0;
        pefile::NT_HEADER = None;
    }
}

fn restore_breakpoints(h_proc: HANDLE, breakpoints: &[u64], offset: u64) {
    for b in breakpoints {
        unsafe {
            memory::breakpoint::restore_byte_of_brkpt(h_proc, *b + offset);
        }
    }
}

fn print_reset_message(item: &str) {
    println!("{VALID_COLOR}all {} have been cleared{RESET_COLOR}", item);
}

pub fn handle_reset(linev: &[&str]) {
    if linev.len() != 2 {
        eprintln!("{}", usage::USAGE_RESET);
        return;
    }
    let binding = linev[1].to_lowercase();
    let opt = binding.trim();
    unsafe {
        match opt {
            "file" => {
                clear_symbols();
                ALL_ELM.file = None;
                print_reset_message("file context");
            }
            "breakpoint" | "b" => {
                ALL_ELM.break_rva.clear();
                print_reset_message("rva breakpoints");
            }
            "symbol" | "s" => {
                clear_symbols();
                print_reset_message("symbols");
            }
            "hook" | "ho" => {
                ALL_ELM.hook.clear();
                print_reset_message("hooks");
            }
            "break-va" | "b-va" => {
                ALL_ELM.break_va.clear();
                print_reset_message("va breakpoints");
            }
            "break-ret" | "b-ret" => {
                ALL_ELM.break_ret.clear();
                print_reset_message("function returns");
            }
            "skip" => {
                ALL_ELM.skip_addr.clear();
                print_reset_message("skipped functions");
            }
            "args" | "arg" | "argv" => {
                ALL_ELM.arg = None;
                print_reset_message("arguments");
            }
            "watchpoint" | "watchpts" | "w" => {
                ALL_ELM.watchpts.clear();
                print_reset_message("watchpoints");
            }
            "all" => {
                clear_symbols();
                ALL_ELM.skip_addr.clear();
                ALL_ELM.break_ret.clear();
                function::FUNC_INFO.clear();
                ALL_ELM.hook.clear();
                *ALL_ELM = All::default();
                print_reset_message("elements");
            }
            _ => eprintln!("{}", usage::USAGE_RESET),
        }
    }
}

pub fn reset_proc(linev: &[&str], h_proc: HANDLE) {
    if linev.len() != 2 {
        eprintln!("{}", usage::USAGE_RESET);
        return;
    }
    let binding = linev[1].to_lowercase();
    let opt = binding.trim();
    unsafe {
        match opt {
            "file" => {
                clear_symbols();
                ALL_ELM.file = None;
                print_reset_message("file context");
            }
            "breakpoint" | "b" | "break" => {
                restore_breakpoints(h_proc, &ALL_ELM.break_rva, BASE_ADDR);
                ALL_ELM.break_rva.clear();
                print_reset_message("breakpoints");
            }
            "break-va" | "b-va" => {
                restore_breakpoints(h_proc, &ALL_ELM.break_va, 0);
                ALL_ELM.break_va.clear();
                print_reset_message("va breakpoints");
            }
            "symbol" | "s" => {
                clear_symbols();
                print_reset_message("symbols");
            }
            "hook" | "ho" => {
                restore_breakpoints(h_proc, &ALL_ELM.hook.iter().map(|h| h.target).collect::<Vec<u64>>(), BASE_ADDR);
                ALL_ELM.hook.clear();
                print_reset_message("hooks");
            }
            "break-ret" | "b-ret" => {
                ALL_ELM.break_ret.clear();
                print_reset_message("function returns");
            }
            "skip" => {
                restore_breakpoints(h_proc, &ALL_ELM.skip_addr, BASE_ADDR);
                ALL_ELM.skip_addr.clear();
                print_reset_message("skipped functions");
            }
            "args" | "arg" | "argv" => {
                ALL_ELM.arg = None;
                print_reset_message("arguments");
            }
            "watchpoint" | "watchpts" | "w" => {
                ALL_ELM.watchpts.clear();
                print_reset_message("watchpoints");
            }
            "all" => {
                clear_symbols();
                restore_breakpoints(h_proc, &ALL_ELM.skip_addr, BASE_ADDR);
                restore_breakpoints(h_proc, &ALL_ELM.break_ret, BASE_ADDR);
                restore_breakpoints(h_proc, &ALL_ELM.hook.iter().map(|h| h.target).collect::<Vec<u64>>(), BASE_ADDR);
                restore_breakpoints(h_proc, &ALL_ELM.break_va, 0);
                restore_breakpoints(h_proc, &ALL_ELM.break_rva, BASE_ADDR);
                *ALL_ELM = All::default();
                print_reset_message("elements");
            }
            _ => eprintln!("{}", usage::USAGE_RESET),
        }
    }
}
