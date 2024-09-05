use winapi::um::winnt::WOW64_CONTEXT;
use crate::usage;
use crate::utils::*;

pub fn handle_set_register(linev: &[&str], ctx: &mut WOW64_CONTEXT) {
    if linev.len() < 3 {
        eprintln!("{}", usage::USAGE_SET_REG);
        return;
    }
    let target = linev[1];
    let value_str = linev[2];
    let value = match str_to::<u32>(value_str) {
        Ok(val) => val,
        Err(e) => {
            eprintln!("{ERR_COLOR}error to parse '{value_str}' : {e}{RESET_COLOR}");
            return;
        }
    };

    match target {
        "eax" => set_reg32(&mut ctx.Eax, value),
        "ebx" => set_reg32(&mut ctx.Ebx, value),
        "ecx" => set_reg32(&mut ctx.Ecx, value),
        "edx" => set_reg32(&mut ctx.Edx, value),
        "esi" => set_reg32(&mut ctx.Esi, value),
        "edi" => set_reg32(&mut ctx.Edi, value),
        "esp" => set_reg32(&mut ctx.Esp, value),
        "ebp" => set_reg32(&mut ctx.Ebp, value),
        "eip" => set_reg32(&mut ctx.Eip, value),
        "flag" | "eflag" => set_reg32(&mut ctx.EFlags, value),
        "cs" => set_reg32(&mut ctx.SegCs, value),
        "ds" => set_reg32(&mut ctx.SegDs, value),
        "es" => set_reg32(&mut ctx.SegEs, value),
        "fs" => set_reg32(&mut ctx.SegFs, value),
        "gs" => set_reg32(&mut ctx.SegGs, value),
        "ss" => set_reg32(&mut ctx.SegSs, value),
        _ => {
            eprintln!("{ERR_COLOR}Unknown register: {}{RESET_COLOR}", target);
            return;
        }
    }
}



fn set_reg32(reg: &mut u32, value: u32) {
    *reg = value;
}