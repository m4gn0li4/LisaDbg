use winapi::um::winnt::WOW64_CONTEXT;
use crate::usage;
use crate::log::*;



pub const ALL_REG32: [&str; 34]  = [
    "eax",
    "ebx",
    "ecx",
    "edx",
    "esi",
    "edi",
    "ebp",
    "esp",
    "eip",
    "cs",
    "segcs",
    "ds",
    "segds",
    "es",
    "seges",
    "fs",
    "segfs",
    "gs",
    "seggs",
    "ss",
    "segss",
    "flag",
    "eflag",
    "ctrl-word",
    "control-word",
    "status-word",
    "tag-word",
    "err-offset",
    "error-offset",
    "err-select",
    "error-selector",
    "data-offset",
    "data-selector",
    "data-select"
];



pub trait ToValue32 {
    fn str_to_ctx(self, target: &str) -> u32;
}



impl ToValue32 for WOW64_CONTEXT {
    fn str_to_ctx(self, target: &str) -> u32 {
        let target = target.to_lowercase();
        match target.as_str() {
            "eax" => self.Eax,
            "ebx" => self.Ebx,
            "ecx" => self.Ecx,
            "edx" => self.Edx,
            "esi" => self.Esi,
            "edi" => self.Edi,
            "ebp" => self.Ebp,
            "esp" => self.Esp,
            "eip" => self.Eip,
            "cs" | "segcs" => self.SegCs,
            "ds" | "segds" => self.SegDs,
            "es" | "seges" => self.SegEs,
            "fs" | "segfs" => self.SegFs,
            "ctrl-word" | "control-word" => self.FloatSave.ControlWord,
            "status-word" => self.FloatSave.StatusWord,
            "tag-word" => self.FloatSave.TagWord,
            "err-offset" | "error-offset" => self.FloatSave.ErrorOffset,
            "err-select" | "error-select" => self.FloatSave.ErrorSelector,
            "data-offset" => self.FloatSave.DataOffset,
            "data-select" => self.FloatSave.DataSelector,
            _ => 0,
        }
    }
}



pub unsafe fn handle_reg(linev: &[&str], ctx: WOW64_CONTEXT) {
    match linev.get(1) {
        Some(&"all-reg") | Some(&"all-register") => {
            for reg_name in ALL_REG32.iter().filter(|&&s|s.len() != 3) {
                let value = ctx.str_to_ctx(reg_name);
                println!("{:<6}: {VALUE_COLOR}{:#x}{RESET_COLOR}", reg_name, value);
            }
        }

        Some(&"all-seg") | Some(&"all-segment") => {
            for reg_name in ALL_REG32.iter().filter(|&reg| reg.ends_with('s') && reg.len() == 2) {
                let value = ctx.str_to_ctx(reg_name);
                println!("{:<3}: {VALUE_COLOR}{:#x}{RESET_COLOR}", reg_name, value);
            }
        }

        Some(&"all") => {
            for reg_name in ALL_REG32 {
                let reg_value = ctx.str_to_ctx(reg_name);
                println!("{}: {VALUE_COLOR}{:#x}{RESET_COLOR}", reg_name, reg_value);
            }
        }

        Some(register) => {
            if let Some(reg_name) = ALL_REG32.iter().find(|&r|r == register) {
                let reg_value = ctx.str_to_ctx(reg_name);
                println!("{:<6}: {VALUE_COLOR}{:#x}{RESET_COLOR}", register, reg_value);
            }
            else {
                println!("{ERR_COLOR}Unknown register: {}{RESET_COLOR}", register);
            }
        }
        None => {
            println!("{}", usage::USAGE_INFO);
        }
    }
}
