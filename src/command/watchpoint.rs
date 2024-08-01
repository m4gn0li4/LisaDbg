use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;
use winapi::um::winnt::{CONTEXT, WOW64_CONTEXT};
use crate::log::{ERR_COLOR, RESET_COLOR, str_to, VALID_COLOR};
use crate::{OPTION, symbol, usage};
use crate::dbg::{BASE_ADDR, memory, RealAddr};
use crate::dbg::dbg_cmd::info_reg::{ToValue, Value};
use crate::dbg::dbg_cmd::mode_32::info_reg::ToValue32;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum FlagTypeMem {
    Stack,
    MemoryStatic,
    VirtualAddr,
    NotDef
}


impl fmt::Display for FlagTypeMem {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            FlagTypeMem::Stack => write!(f, "stack"),
            FlagTypeMem::MemoryStatic => write!(f, "static memory"),
            FlagTypeMem::VirtualAddr => write!(f, "Absolute address"),
            _ => write!(f, ""),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
#[derive(Hash)]
pub enum CheckType {
    X,
    W,
    R,
}


impl fmt::Display for CheckType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CheckType::X => write!(f, "execute"),
            CheckType::W => write!(f, "write"),
            CheckType::R => write!(f, "read"),
        }
    }
}



#[derive(Debug, Clone)]
pub struct Watchpts {
    pub offset: i64,
    pub flag_type_mem: FlagTypeMem,
    pub check_type: Vec<CheckType>,
    pub memory_size: usize,
    pub register: String,
}



impl RealAddr for Watchpts {
    fn real_addr64(&self, ctx: CONTEXT) -> u64 {
        unsafe {
            if self.register != "" {
                let value = ctx.str_to_value_ctx(&self.register);
                match value {
                    Value::U64(value_reg) => return (value_reg as i64 + self.offset) as u64,
                    Value::U128(_) => eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> cannot take the value of a register 128 as a basis"),
                    Value::Un => eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> unknow register : {}", self.register)
                }
                return 0;
            }
            match self.flag_type_mem {
                FlagTypeMem::Stack => {
                    if let Some(frame) = memory::stack::get_frame_before_func(ctx.Rip) {
                        ((frame.AddrStack.Offset as u64) as i64 + self.offset) as u64
                    }else {
                        0
                    }
                },
                FlagTypeMem::MemoryStatic => self.offset as u64 + BASE_ADDR,
                FlagTypeMem::VirtualAddr => self.offset as u64,
                _ => 0
            }
        }
    }

    fn real_addr32(&self, ctx: WOW64_CONTEXT) -> u32 {
        unsafe {
            if self.register != "" {
                return ctx.str_to_ctx(&self.register)
            }
            match self.flag_type_mem {
                FlagTypeMem::Stack => {
                    if let Some(frame) = memory::stack::get_frame_before_func(ctx.Eip as u64) {
                        (frame.AddrStack.Offset as i64 + self.offset) as u32
                    }else {
                        0
                    }
                },
                FlagTypeMem::MemoryStatic => self.offset as u32 + BASE_ADDR as u32,
                FlagTypeMem::VirtualAddr => self.offset as u32,
                _ => 0
            }
        }
    }
}


impl Watchpts {
    pub fn acces_type_to_bits(&self) -> u32{
        let mut result = 0;
        for ac_type in &self.check_type {
            match ac_type {
                CheckType::R => result |= 1 << 0,
                CheckType::W => result |= 1 << 1,
                CheckType::X => result |= 1 << 2,
            }
        }
        return result;
    }


    pub fn format_offset(&self, ctx: CONTEXT) -> String {
        return if unsafe {BASE_ADDR != 0} || self.flag_type_mem == FlagTypeMem::VirtualAddr{
            format!("{:#x}", self.real_addr64(ctx))
        }else if self.flag_type_mem == FlagTypeMem::Stack{
            format!(".fp{:+}", self.offset)
        }else {
            format!("base-address + {:#x}", self.offset)
        }
    }
}





impl FromStr for Watchpts {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        let mut offset: Option<i64> = None;
        let mut flag_type_mem = FlagTypeMem::NotDef;
        let mut check_type = vec![CheckType::W, CheckType::R];
        let mut memory_size = usize::MAX;
        let mut register = String::new();
        for (i, part) in parts.iter().enumerate() {
            if part.starts_with("--memory=") {
                let mem_type = part.trim_start_matches("--memory=").to_lowercase();
                flag_type_mem = match mem_type.as_str() {
                    "stack" => FlagTypeMem::Stack,
                    "static" | "static-mem" | "static-memory" => FlagTypeMem::MemoryStatic,
                    "virtual" | "virtual-addr" | "virtual-address" => FlagTypeMem::VirtualAddr,
                    _ => return Err(format!("{ERR_COLOR}Invalid memory type: {mem_type}{RESET_COLOR}")),
                };
            } else if part.starts_with("--access=") {
                let access_rights = part.trim_start_matches("--access=");
                check_type = access_rights
                    .chars()
                    .map(|c| match c {
                        'x' | 'X' => Ok(CheckType::X),
                        'w' | 'W' => Ok(CheckType::W),
                        'r' | 'R' => Ok(CheckType::R),
                        _ => Err(format!("{ERR_COLOR}Invalid access type : {c}{RESET_COLOR}")),
                    }).collect::<Result<Vec<CheckType>, _>>()?;

                let mut seen = std::collections::HashSet::new();
                for ct in &check_type {
                    if !seen.insert(ct) {
                        return Err(format!("{ERR_COLOR}you cannot specify the same access rights twice : {ct}{RESET_COLOR}"));
                    }
                }
            } else if part.starts_with("--size") {
                if let Some(size_str) = parts.get(i+1) {
                    match str_to::<usize>(size_str) {
                        Ok(size) => memory_size = size,
                        Err(e) => return Err(format!("{ERR_COLOR}invalid entry : {e}{RESET_COLOR}")),
                    }
                }else {
                    return Err(format!("{ERR_COLOR}you did not specify a arg for --size{RESET_COLOR}"))
                }
            }else if part.starts_with("--reg=") | part.starts_with("--register=") {
                register = part.trim_start_matches("--reg=").trim_start_matches("--register=").to_string();
            }
            else {
                match str_to::<i64>(part) {
                    Ok(parsed_offset) => offset = Some(parsed_offset),
                    Err(_) => {
                        if let Some(sym) = unsafe {symbol::SYMBOLS_V.symbol_file.iter().find(|s|s.name == part.trim_start().trim_end())} {
                            if flag_type_mem == FlagTypeMem::NotDef && sym.offset < 0 {
                                flag_type_mem = FlagTypeMem::Stack;
                            }
                            offset = Some(sym.offset);
                            if memory_size == usize::MAX {
                                memory_size = sym.size;
                            }
                            if sym.register != 0 {
                                register = symbol::pdb::get_reg_with_reg_field(sym.register);
                            }
                        }else {
                            return Err(format!("{ERR_COLOR}Invalid part in watchpoint format : {part}{RESET_COLOR}"));
                        }
                    }
                }
            }
        }
        let offset = offset.ok_or(format!("{ERR_COLOR}Missing offset value{RESET_COLOR}"))?;
        if flag_type_mem == FlagTypeMem::NotDef {
            flag_type_mem = FlagTypeMem::MemoryStatic;
        }
        if memory_size == usize::MAX {
            memory_size = 0;
        }
        Ok(Watchpts {offset, flag_type_mem, check_type, memory_size, register})
    }
}



pub fn watchpoint(linev: &[&str]) {
    if linev.len() == 1 {
        println!("{}", usage::USAGE_WATCHPTS);
        return;
    }
    unsafe {
        match Watchpts::from_str(&linev[1..].join(" ")) {
            Ok(wt) =>  {
                if OPTION.watchpts.len() < 4 {
                    OPTION.watchpts.push(wt.clone());
                    println!("{VALID_COLOR}watchpoint was set successfully at offset {}: {}{RESET_COLOR}",wt.format_offset(std::mem::zeroed()), OPTION.watchpts.len())
                }else {
                    eprintln!("{ERR_COLOR}you can only place 4 watchpoints{RESET_COLOR}");
                }
            },
            Err(e) => eprintln!("{e}"),
        }
    }
}



pub fn watchpoint_proc(linev: &[&str], ctx: &mut CONTEXT) {
    if linev.len() == 1 {
        println!("{}", usage::USAGE_WATCHPTS);
        return;
    }
    match Watchpts::from_str(&linev[1..].join(" ")) {
        Ok(wt) => unsafe {
            if OPTION.watchpts.len() < 4 {
                OPTION.watchpts.push(wt.clone());
                memory::watchpoint::set_dreg(ctx, &wt, OPTION.watchpts.len()-1);
                println!("{VALID_COLOR}watchpoint {} was set at successfully for watch address {:#x} with access {:?}{RESET_COLOR}", OPTION.watchpts.len(), wt.real_addr64(*ctx), wt.check_type);
            }else {
                eprintln!("{ERR_COLOR}you can only place 4 watchpoints{RESET_COLOR}");
            }
        },
        Err(e) => eprintln!("{e}"),
    }
}
