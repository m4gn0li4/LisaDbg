use std::ffi::{c_char, CStr, CString};
use libloading::{Library, Symbol};
use crate::{OPTION, symbol};


#[repr(C)]
#[derive(Debug)]
struct Symbols {
    size: u32,
    value: u64,
    address: u64,
    tag: u32,
    name: *const c_char,
}


pub unsafe fn target_symbol() {
    let dll = Library::new("symbol_pe.dll").expect("the dll 'symbol_pe.dll' was not found");
    let get_vector: Symbol<unsafe extern "C" fn(*mut usize, *const c_char) -> *mut Symbols> = dll.get(b"symbole").expect("Failed to get 'symbole' function in dll 'symbol_pe.dll'");
    let mut length = 0;
    let path = match CString::new(OPTION.file.clone().unwrap()) {
        Ok(path) => path,
        Err(_) => return,
    };
    let begin_vec = get_vector(&mut length, path.as_ptr());
    if !begin_vec.is_null(){
        symbol::SYMBOLS_V.symbol_type = symbol::SymbolType::PDB;
    }else {
        return
    }
    let vec_slice = std::slice::from_raw_parts(begin_vec, length);
    let get_tag_str: Symbol<unsafe extern "C" fn(u32) -> *const c_char> = dll.get(b"GetTagString").expect("failed to get 'GetTagString' function in dll 'symbol_pe.dll'");
    for sym in vec_slice {
        let mut sym_e = symbol::SymbolFile::default();
        if symbol::IMAGE_BASE < sym.address {
            sym_e.addr = sym.address - symbol::IMAGE_BASE;
        }else {
            sym_e.addr = sym.address;
        }
        sym_e.size = sym.size as usize;
        sym_e.types_e = CStr::from_ptr(get_tag_str(sym.tag)).to_string_lossy().to_string();
        sym_e.name = CStr::from_ptr(sym.name).to_string_lossy().to_string();
        symbol::SYMBOLS_V.symbol_file.push(sym_e);
    }
    let free_symbols: Symbol<unsafe extern "C" fn()> = dll.get(b"free_symbols").expect("failed to get 'free_symbols' function in dll 'symbol_pe.dll'");
    free_symbols();
}