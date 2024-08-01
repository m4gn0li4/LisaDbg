use crate::log::*;
use regex::Regex;
use winapi::shared::ntdef::HANDLE;
use winapi::um::winnt::{WOW64_CONTEXT};
use crate::dbg::dbg_cmd::usages;
use crate::log::str_to;

pub fn handle_set_memory(process_handle: HANDLE, ctx: WOW64_CONTEXT, linev: &[&str]) {
    if linev.len() < 4 {
        eprintln!("{}", usages::USAGE_SET_MEM);
        return;
    }
    let types = linev[1];
    let target = linev[2];
    let new_value_str = linev[3..].join(" ");
    let mut size = 1;
    let re = Regex::new(r"\[(.*?)]").unwrap();
    for cap in re.captures_iter(types) {
        if let Some(numd) = cap.get(1) {
            match str_to::<usize>(numd.as_str()) {
                Ok(num) => size = num,
                Err(e) => if e.to_string().contains("empty string") {
                    size = usize::MAX;
                }
            }
        }
    }
    let mut target_addr = reg32_to_value(target, &ctx) as u64;
    if target_addr == 0 {
        match str_to::<u32>(target) {
            Ok(addr) => target_addr = addr as u64,
            Err(e) =>  {
                eprintln!("{ERR_COLOR}invalid target address : {e}{RESET_COLOR}");
                return;
            }
        }
    }

    let types_r = types.split('[').next().unwrap_or_default();
    match types_r {
        "uint8_t" | "char" => crate::dbg::dbg_cmd::modifier::set_memory::target_in_memory::<u8>(process_handle, &new_value_str, target_addr, size),
        "int8_t" => crate::dbg::dbg_cmd::modifier::set_memory::target_in_memory::<i8>(process_handle, &new_value_str, target_addr, size),
        "uint16_t" => crate::dbg::dbg_cmd::modifier::set_memory::target_in_memory::<u16>(process_handle, &new_value_str, target_addr, size),
        "int16_t" => crate::dbg::dbg_cmd::modifier::set_memory::target_in_memory::<i16>(process_handle, &new_value_str, target_addr, size),
        "uint32_t" => crate::dbg::dbg_cmd::modifier::set_memory::target_in_memory::<u32>(process_handle, &new_value_str, target_addr, size),
        "int32_t" => crate::dbg::dbg_cmd::modifier::set_memory::target_in_memory::<i32>(process_handle, &new_value_str, target_addr, size),
        "uint64_t" => crate::dbg::dbg_cmd::modifier::set_memory::target_in_memory::<u64>(process_handle, &new_value_str, target_addr, size),
        "int64_t" => crate::dbg::dbg_cmd::modifier::set_memory::target_in_memory::<i64>(process_handle, &new_value_str, target_addr, size),
        _ => eprintln!("{ERR_COLOR}unsupported type{RESET_COLOR}"),
    }
}



pub fn reg32_to_value(reg: &str, ctx: &WOW64_CONTEXT) -> u32 {
    match reg {
        "eax" => ctx.Eax,
        "ebx" => ctx.Ebx,
        "ecx" => ctx.Ecx,
        "edx" => ctx.Edx,
        "eip" => ctx.Eip,
        "esi" => ctx.Esi,
        "ebp" => ctx.Ebp,
        "esp" => ctx.Esp,
        "edi" => ctx.Edi,
        _ => 0,
    }
}