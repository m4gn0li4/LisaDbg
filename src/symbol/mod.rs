pub mod pdb;
mod dwarf;

use std::cmp::PartialEq;
use std::fmt;
use std::fmt::Formatter;
use once_cell::sync::Lazy;
use crate::{OPTION, pefile};
use crate::log::*;


#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SymbolType {
    DWARF,
    PDB,
    Un
}


impl fmt::Display for SymbolType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SymbolType::Un => write!(f, "unknow"),
            SymbolType::DWARF => write!(f, "DWARF"),
            SymbolType::PDB => write!(f, "PDB"),
        }
    }
}



impl Default for SymbolType {
    fn default() -> Self { SymbolType::Un }
}



#[derive(Debug, Default, Eq, PartialEq)]
pub struct SymbolFile {
    pub name: String,
    pub addr: u64,
    pub size: usize,
    pub value_str: String,
    pub types_e: String
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



pub fn target_addr_with_name_sym(name: &str) -> u64 {
    unsafe {
        SYMBOLS_V.symbol_file
            .iter()
            .find(|sym| sym.addr != 0 && sym.name.eq_ignore_ascii_case(name))
            .map_or(0, |sym| sym.addr)
    }
}