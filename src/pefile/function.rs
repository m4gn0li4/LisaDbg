use winapi::um::winnt::{IMAGE_DATA_DIRECTORY, RUNTIME_FUNCTION};
use std::{mem, slice};
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


pub fn parse_pdata(pdata_dir: IMAGE_DATA_DIRECTORY) {
    if pdata_dir.VirtualAddress == 0 || pdata_dir.Size == 0 {
        eprintln!("{ERR_COLOR}no section is IMAGE_DIRECTORY_ENTRY_EXCEPTION{RESET_COLOR}");
        return;
    }
    let rva_pdata = pdata_dir.VirtualAddress;
    for section in unsafe { &*pefile::section::SECTION_VS } {
        if rva_pdata == section.addr {
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