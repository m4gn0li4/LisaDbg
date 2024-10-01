use crate::{symbol, ALL_ELM};
use std::ffi::{c_char, CStr, CString};
use std::mem;
use winapi::um::libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryA};

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
    let symbol_pe = LoadLibraryA("symbol_pe.dll\0".as_ptr() as *const i8);
    if symbol_pe.is_null() {
        panic!("failed to load symbol_pe.dll");
    }
    let get_vector: unsafe extern "C" fn(&mut usize, *const c_char) -> *mut SymbolsPdb = mem::transmute(GetProcAddress(symbol_pe, "getSymbols\0".as_ptr() as *const i8));

    let mut length = 0;
    let path = CString::new(ALL_ELM.file.clone().unwrap()).unwrap();
    let begin_vec = get_vector(&mut length, path.as_ptr());
    if length != 0 {
        symbol::SYMBOLS_V.symbol_type = symbol::SymbolType::PDB;
    } else {
        return;
    }
    let vec_slice = std::slice::from_raw_parts(begin_vec, length);
    let get_tag_str: unsafe extern "C" fn(u32) -> *const c_char = mem::transmute(GetProcAddress(symbol_pe, "GetTagString\0".as_ptr() as *const i8));
    for sym in vec_slice {
        let mut sym_e = symbol::SymbolFile::default();
        if symbol::IMAGE_BASE < sym.address {
            sym_e.offset = (sym.address - symbol::IMAGE_BASE) as i64;
        } else {
            sym_e.offset = sym.address as i64;
        }
        sym_e.size = sym.size as usize;
        sym_e.types_e = CStr::from_ptr(get_tag_str(sym.tag)).to_string_lossy().to_string();
        sym_e.name = CStr::from_ptr(sym.name).to_string_lossy().to_string();
        sym_e.filename = CStr::from_ptr(sym.filename).to_string_lossy().to_string();
        sym_e.line = sym.line as usize;
        symbol::SYMBOLS_V.symbol_file.push(sym_e);
    }
    let free_symbols: unsafe extern "C" fn(*mut SymbolsPdb, usize) = mem::transmute(GetProcAddress(symbol_pe, "freeSymbols\0".as_ptr() as *const i8));
    free_symbols(begin_vec, length);
    FreeLibrary(symbol_pe);
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
