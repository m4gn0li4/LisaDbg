pub mod pdb;
mod dwarf;

use std::cmp::PartialEq;
use std::fmt;
use std::fmt::Formatter;
use libloading::Library;
use once_cell::sync::Lazy;
use winapi::um::winnt::{CONTEXT, WOW64_CONTEXT};
use crate::{OPTION, pefile};
use crate::dbg::dbg_cmd::info_reg::{ToValue, Value};
use crate::dbg::{BASE_ADDR, memory, RealAddr};
use crate::dbg::dbg_cmd::mode_32::info_reg::ToValue32;
use crate::log::*;




pub static SYMBOL_PE: Lazy<Library> = Lazy::new(|| unsafe { Library::new("symbol_pe.dll") }.expect("the dll 'symbol_pe.dll' was not found"));


#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SymbolType {
    DWARF,
    PDB,
    Un
}




impl fmt::Display for SymbolType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SymbolType::Un => write!(f, "UNKNOW"),
            SymbolType::DWARF => write!(f, "DWARF"),
            SymbolType::PDB => write!(f, "PDB"),
        }
    }
}



impl Default for SymbolType {
    fn default() -> Self { SymbolType::Un }
}


#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct SymbolFile {
    pub name: String,
    pub offset: i64,
    pub size: usize,
    pub value_str: String,
    pub types_e: String,
    pub filename: String,
    pub line: usize,
    pub register: u32,
}




impl RealAddr for SymbolFile {
    fn real_addr64(&self, ctx: CONTEXT) -> u64 {
        if self.register != 0 {
            let value = ctx.str_to_value_ctx(&pdb::get_reg_with_reg_field(self.register));
            return match value {
                Value::U64(reg_value) => (reg_value as i64 + self.offset) as u64,
                _ => {
                    eprintln!("{ERR_COLOR}invalid register : {}{RESET_COLOR}", format!("{} ({})", self.register, &pdb::get_reg_with_reg_field(self.register)));
                    0
                }
            }
        }
        if self.offset < 0 {
            unsafe {
                if let Some(b_frame) = memory::stack::get_frame_before_func(ctx.Rip) {
                    return (b_frame.AddrStack.Offset as i64 + self.offset) as u64
                }else {
                    eprintln!("{ERR_COLOR}failed to get last frame before current frame{RESET_COLOR}");
                    0
                }
            }
        }else {
            unsafe { return BASE_ADDR + self.offset as u64 }
        }
    }


    fn real_addr32(&self, ctx: WOW64_CONTEXT) -> u32 {
        if self.register != 0 {
            return ctx.str_to_ctx(&pdb::get_reg_with_reg_field(self.register));
        }
        if self.offset < 0 {
            unsafe {
                if let Some(b_frame) = memory::stack::get_frame_before_func(ctx.Eip as u64) {
                    return (b_frame.AddrStack.Offset as i64 + self.offset) as u32
                }else {
                    eprintln!("{ERR_COLOR}failed to get last frame before current frame{RESET_COLOR}");
                    0
                }
            }
        }else {
            unsafe { return (BASE_ADDR + self.offset as u64) as u32}
        }
    }
}





#[derive(Debug, Default, Eq, PartialEq)]
pub struct Symbols {
    pub symbol_type: SymbolType,
    pub symbol_file: Vec<SymbolFile>
}



pub static mut SYMBOLS_V: Lazy<Symbols> = Lazy::new(||Symbols::default());
pub static mut IMAGE_BASE: u64 = 0;


pub fn load_symbol() {
    unsafe {
        if OPTION.file.is_none(){
            eprintln!("{ERR_COLOR}you must first specify a file{RESET_COLOR}");
            return
        }
        if let Err(e) = dwarf::target_dwarf_info(&*pefile::section::SECTION_VS) {
            eprintln!("{ERR_COLOR}Error target symbol dwarf : {e}{RESET_COLOR}");
            return;
        }
        pdb::target_symbol();
        if SYMBOLS_V.symbol_type != SymbolType::Un {
            println!("{VALID_COLOR}the symbol file was loaded with success\nsymbol type : {}{RESET_COLOR}", SYMBOLS_V.symbol_type)
        }else {
            eprintln!("the file does not contain a supported symbol format")
        }
    }
}


