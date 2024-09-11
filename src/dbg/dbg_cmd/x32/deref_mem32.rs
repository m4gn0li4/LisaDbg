use winapi::shared::ntdef::HANDLE;
use winapi::um::winnt::{WOW64_CONTEXT};
use crate::dbg::dbg_cmd::usages;
use crate::dbg::dbg_cmd::x32::info_reg::ToValue32;
use crate::dbg::memory::deref_mem;
use crate::utils::*;


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
        ctx.str_to_ctx(target)
    };

    if address == 0 {
        eprintln!("{ERR_COLOR}invalid register or null address{RESET_COLOR}");
        return;
    }
    if let Err(err) = deref_mem::deref_memory(process_handle, dtype, address as usize) { eprintln!("{ERR_COLOR}{}{RESET_COLOR}", err) }
}
