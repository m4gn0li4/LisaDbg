use std::ffi::{c_char, CStr, CString};
use libloading::Symbol;
use crate::{OPTION, symbol};
use crate::symbol::SYMBOL_PE;

#[repr(C)]
#[derive(Debug)]
pub struct SymbolsPdb {
    pub size: u32,
    pub value: u64,
    pub address: u64,
    pub tag: u32,
    pub name: *const c_char,
    pub filename: *const c_char,
    pub line: u32,
}


pub unsafe fn target_symbol() {
    let get_vector: Symbol<unsafe extern "C" fn(&mut usize, *const c_char) -> *mut SymbolsPdb> = SYMBOL_PE.get(b"getSymbols").expect("Failed to get 'symbole' function in dll 'symbol_pe.dll'");
    let mut length = 0;
    let path = CString::new(OPTION.file.clone().unwrap()).unwrap();
    let begin_vec = get_vector(&mut length, path.as_ptr());
    if length != 0 {
        symbol::SYMBOLS_V.symbol_type = symbol::SymbolType::PDB;
    }else {
        return
    }
    let vec_slice = std::slice::from_raw_parts(begin_vec, length);
    let get_tag_str: Symbol<unsafe extern "C" fn(u32) -> *const c_char> = SYMBOL_PE.get(b"GetTagString").expect("failed to get 'GetTagString' function in dll 'symbol_pe.dll'");
    for sym in vec_slice {
        let mut sym_e = symbol::SymbolFile::default();
        if symbol::IMAGE_BASE < sym.address {
            sym_e.offset = (sym.address - symbol::IMAGE_BASE) as i64;
        }else {
            sym_e.offset = sym.address as i64;
        }
        sym_e.size = sym.size as usize;
        sym_e.types_e = CStr::from_ptr(get_tag_str(sym.tag)).to_string_lossy().to_string();
        sym_e.name = CStr::from_ptr(sym.name).to_string_lossy().to_string();
        sym_e.filename = CStr::from_ptr(sym.filename).to_string_lossy().to_string();
        sym_e.line = sym.line as usize;
        symbol::SYMBOLS_V.symbol_file.push(sym_e);
    }
    let free_symbols: Symbol<unsafe extern "C" fn(*mut SymbolsPdb, usize)> = SYMBOL_PE.get(b"freeSymbols").expect("failed to get 'free_symbols' function in dll 'symbol_pe.dll'");
    free_symbols(begin_vec, length);
}




pub fn get_reg_with_reg_field(reg_field: u32) -> String {
    let res = match reg_field {
        17 => "eax",
        18 => "ecx",
        19 => "edx",
        328 => "rax",
        329 => "rbx",
        330 => "rcx",
        331 => "rdx",
        332 => "rsi",
        333 => "rdi",
        334 => "rbp",
        335 => "rsp",
        336 => "r8",
        337 => "r9",
        338 => "r10",
        339 => "r11",
        340 => "r12",
        341 => "r13",
        342 => "r14",
        343 => "r15",
        _ => "",
    };
    res.to_string()
}