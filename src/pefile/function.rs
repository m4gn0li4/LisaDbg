use winapi::um::winnt::{IMAGE_DATA_DIRECTORY, RUNTIME_FUNCTION};
use std::{mem, slice};
use crate::utils::*;
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
        eprintln!("{WAR_COLOR}no section is IMAGE_DIRECTORY_ENTRY_EXCEPTION{RESET_COLOR}");
        return;
    }
    let rva_pdata = pdata_dir.VirtualAddress;
    for section in unsafe { &*pefile::section::SECTION_VS } {
        if section.addr <= rva_pdata && section.addr + section.content.len() as u32 >= pdata_dir.VirtualAddress + pdata_dir.Size {
            let runt_size = section.content.len() / mem::size_of::<RUNTIME_FUNCTION>();
            let base_pdata = section.content.as_ptr() as *const RUNTIME_FUNCTION;
            let mut runt_func = unsafe { slice::from_raw_parts(base_pdata, runt_size) }.to_vec();
            runt_func.retain(|f|f.BeginAddress != 0);
            unsafe {
                FUNC_INFO.clear();
                FUNC_INFO.extend_from_slice(&runt_func);
            }
            return;
        }
    }
    eprintln!("{ERR_COLOR}no section is IMAGE_DIRECTORY_ENTRY_EXCEPTION{RESET_COLOR}")
}