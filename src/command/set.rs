use crate::dbg::dbg_cmd::x32;
use crate::dbg::dbg_cmd::x64::modifier;
use crate::dbg::memory::set;
use crate::dbg::memory::set::set_memory::set_memory64;
use crate::usage;
use crate::utils::ERR_COLOR;
use winapi::um::winnt::{CONTEXT, HANDLE, WOW64_CONTEXT};

pub fn set_element64(h_proc: HANDLE, ctx: &mut CONTEXT, linev: &[&str]) {
    if linev.len() < 3 {
        println!("{}", usage::USAGE_SET);
        return;
    }
    let type_set = linev[1].to_lowercase();
    let target = &linev[2..];
    match type_set.as_str() {
        "memory" | "mem" => set_memory64(h_proc, *ctx, target),
        "mem-protect" | "memory-protect" => set::set_protect::change_protect64(h_proc, *ctx, target),
        "register" | "reg" => modifier::register::set_register64(&target, ctx),
        _ => eprintln!("{ERR_COLOR}unknow element {}", linev[1]),
    }
}

pub fn set_element32(h_proc: HANDLE, ctx: &mut WOW64_CONTEXT, linev: &[&str]) {
    if linev.len() < 3 {
        println!("{}", usage::USAGE_SET);
        return;
    }
    let type_set = linev[1].to_lowercase();
    let target = &linev[2..];
    match type_set.as_str() {
        "memory" | "mem" => set::set_memory::set_memory32(h_proc, *ctx, target),
        "mem-protect" | "memory-protect" => set::set_protect::change_protect32(h_proc, *ctx, target),
        "register" | "reg" => x32::modifier32::register::set_register32(&target, ctx),
        _ => eprintln!("{ERR_COLOR}unknow element {}", linev[1]),
    }
}
