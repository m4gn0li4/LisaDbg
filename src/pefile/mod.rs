pub mod section;
pub mod function;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::{io, slice};
use std::ptr::addr_of_mut;
use winapi::um::winnt::{IMAGE_DOS_HEADER, IMAGE_DOS_SIGNATURE, IMAGE_NT_HEADERS, IMAGE_NT_HEADERS64};
use crate::OPTION;


pub struct Section {
    pub name: String,
    pub content: Vec<u8>,
    pub addr: usize,
}



pub static mut NT_HEADER: IMAGE_NT_HEADERS64 = unsafe {std::mem::zeroed()};



pub fn get_name(smptroffs: u64, file: &mut File, name_bytes: Vec<u8>) -> Vec<u8>{
    if let Some(index) = name_bytes.iter().position(|&b| b == b'/') {
        let offset_str = std::str::from_utf8(&name_bytes[index + 1..]).unwrap();
        let cleaned_insn: String = offset_str.chars().filter(|&c| c.is_digit(10) || c == '-').collect();
        let offset: u64 = cleaned_insn.parse().unwrap();
        let mut table_string = vec![0u8; 256];
        file.seek(SeekFrom::Start(offset + smptroffs)).unwrap();
        file.read_exact(&mut table_string).unwrap();
        let name_length = table_string.iter().position(|&b| b == 0).unwrap_or(256);
        [&name_bytes[..index], &table_string[..name_length]].concat()
    }else {
        name_bytes
    }
}



pub unsafe fn parse_header() -> Result<(), io::Error>{
    let mut file = File::open(unsafe {OPTION.file.clone().unwrap()}).unwrap();
    let mut dos_header: IMAGE_DOS_HEADER = unsafe { std::mem::zeroed() };
    file.read_exact(slice::from_raw_parts_mut(&mut dos_header as *mut _ as *mut u8, std::mem::size_of_val(&dos_header)))?;
    if dos_header.e_magic != IMAGE_DOS_SIGNATURE {
        return Err(io::Error::new(io::ErrorKind::Other, "Invalid DOS signature"));
    }
    file.seek(SeekFrom::Start(dos_header.e_lfanew as u64))?;
    file.read_exact(slice::from_raw_parts_mut(addr_of_mut!(NT_HEADER) as *mut u8, std::mem::size_of::<IMAGE_NT_HEADERS>()))?;
    crate::symbol::IMAGE_BASE = NT_HEADER.OptionalHeader.ImageBase;
    section::parse_section(NT_HEADER, &mut file)?;
    function::parse_pdata(NT_HEADER.OptionalHeader.DataDirectory[3]);
    Ok(())
}