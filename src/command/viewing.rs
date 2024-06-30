use std::ptr::addr_of;
use crate::{OPTION, usage};
use crate::log::{ERR_COLOR, RESET_COLOR, VALUE_COLOR};
use crate::ste::{ST_OVER_ADDR, STE_RETURN_ADDR};

pub fn view_brpkt(linev: &[&str]) {
    if linev.len() != 2 {
        println!("{}", usage::USAGE_VIEW);
        return;
    }
    let elm = linev[1];
    match elm {
        "breakpoint" | "brpt" | "b" => print_elements(unsafe { &*addr_of!(OPTION.breakpoint_addr) }, "breakpoint"),
        "skip" => print_elements(unsafe { &*addr_of!(ST_OVER_ADDR) }, "skip"),
        "stret" => print_elements(unsafe { &*addr_of!(STE_RETURN_ADDR) }, "stret"),
        _ => eprintln!("{ERR_COLOR}unknow option : '{elm}'{RESET_COLOR}"),
    }
}

fn print_elements(elements: &[u64], _type: &str) {
    for (i, e) in elements.iter().enumerate() {
        println!("{i} : {VALUE_COLOR}{:#x}{RESET_COLOR}", e);
    }
}
