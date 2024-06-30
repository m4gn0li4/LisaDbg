use winapi::um::winnt::{IMAGE_DATA_DIRECTORY, RUNTIME_FUNCTION};
use std::{mem, slice};
use crate::dbg::BASE_ADDR;
use crate::log::*;
use crate::pefile;


pub static mut FUNC_INFO: Vec<RUNTIME_FUNCTION> = Vec::new();



#[derive(Default, Clone)]
pub struct CrtFunc {
    pub name: String,
    pub ret_value: u64,
    pub address: u64,
}


pub static mut CR_FUNCTION: Vec<CrtFunc> = Vec::new();

pub trait Verifie {
    fn is_in_func(&self, begin_addr: u64, addr: u64) -> bool;
}


impl Verifie for Vec<RUNTIME_FUNCTION> {
    fn is_in_func(&self, begin_addr: u64, addr: u64) -> bool {
        let begin_addr = (begin_addr - unsafe {BASE_ADDR}) as u32;
        let addr = (addr - unsafe {BASE_ADDR}) as u32;
        for func_info in self {
            if begin_addr == func_info.BeginAddress && func_info.BeginAddress <= addr && func_info.EndAddress >= addr {
                return true;
            }
        }
        false
    }
}


pub fn parse_pdata(pdata_dir: IMAGE_DATA_DIRECTORY) {
    if pdata_dir.VirtualAddress == 0 || pdata_dir.Size == 0 {
        return;
    }
    let rva_pdata = pdata_dir.VirtualAddress as usize;
    for section in unsafe { &*pefile::section::SECTION_VS } {
        let section_start = section.addr;
        if rva_pdata == section_start {
            let runt_size = section.content.len() / mem::size_of::<RUNTIME_FUNCTION>();
            let base_pdata = section.content.as_ptr() as *const RUNTIME_FUNCTION;
            let runt_func = unsafe { slice::from_raw_parts(base_pdata, runt_size) };
            unsafe {
                FUNC_INFO.clear();
                FUNC_INFO.extend_from_slice(runt_func);
            }
            return;
        }
    }
    eprintln!("{ERR_COLOR}no section is IMAGE_DIRECTORY_ENTRY_EXCEPTION{RESET_COLOR}")
}