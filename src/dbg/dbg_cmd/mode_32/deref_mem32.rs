use winapi::shared::ntdef::HANDLE;
use winapi::um::winnt::{WOW64_CONTEXT};
use crate::dbg::dbg_cmd::mode_32::modifier32::set_mem;
use crate::dbg::dbg_cmd::usages;
use crate::log::*;


pub fn handle_deref32(linev: &[&str], ctx: WOW64_CONTEXT, process_handle: HANDLE) {
    if linev.len() < 3 {
        eprintln!("{}", usages::USAGE_DEREF);
        return
    }
    let dtype = linev[1];
    let target = linev[2];
    let address = if let Ok(addr) = str_to::<u32>(target) {
        addr
    } else {
        set_mem::reg32_to_value(target, &ctx)
    };

    if address == 0 {
        eprintln!("{ERR_COLOR}invalid register or null address{RESET_COLOR}");
        return;
    }
    if let Err(err) = crate::dbg::dbg_cmd::deref_mem::deref_memory(process_handle, dtype, address as usize) { eprintln!("{ERR_COLOR}{}{RESET_COLOR}", err) }
}
