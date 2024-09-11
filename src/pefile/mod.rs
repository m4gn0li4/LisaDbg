pub mod section;
pub mod function;

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::{io, slice};
use winapi::um::winnt::{IMAGE_DOS_HEADER, IMAGE_NT_HEADERS64, IMAGE_FILE_HEADER, IMAGE_NT_HEADERS32};


use crate::OPTION;

pub struct Section {
    pub name: String,
    pub content: Vec<u8>,
    pub addr: u32,
    pub ptr2raw: u32
}



#[derive(Clone, Copy)]
pub enum NtHeaders {
    Headers32(IMAGE_NT_HEADERS32),
    Headers64(IMAGE_NT_HEADERS64),
}

impl NtHeaders {
    pub fn get_bitness(self) -> usize {
        match self {
            NtHeaders::Headers32(_) => 32,
            NtHeaders::Headers64(_) => 64,
        }
    }

    pub fn get_size_of_arch(self) -> usize {
        match self {
            NtHeaders::Headers32(_) => 4,
            NtHeaders::Headers64(_) => 8,
        }
    }
}



pub static mut NT_HEADER: Option<NtHeaders> = None;

pub fn get_name(smptroffs: u64, file: &mut File, name_bytes: Vec<u8>) -> Vec<u8> {
    if let Some(index) = name_bytes.iter().position(|&b| b == b'/') {
        let offset_str = std::str::from_utf8(&name_bytes[index + 1..]).unwrap();
        let cleaned_insn: String = offset_str.chars().filter(|&c| c.is_digit(10) || c == '-').collect();
        let offset: u64 = cleaned_insn.parse().unwrap();
        let mut table_string = vec![0u8; 256];
        file.seek(SeekFrom::Start(offset + smptroffs)).unwrap();
        file.read_exact(&mut table_string).unwrap();
        let name_length = table_string.iter().position(|&b| b == 0).unwrap_or(256);
        [&name_bytes[..index], &table_string[..name_length]].concat()
    } else {
        name_bytes
    }
}




pub unsafe fn parse_header() -> Result<(), io::Error> {
    let mut file = File::open(OPTION.file.clone().unwrap())?;
    let mut dos_header: IMAGE_DOS_HEADER = std::mem::zeroed();
    file.read_exact(slice::from_raw_parts_mut(&mut dos_header as *mut _ as *mut u8, std::mem::size_of::<IMAGE_DOS_HEADER>()))?;
    if dos_header.e_magic != 0x5a4d {
        return Err(io::Error::new(io::ErrorKind::Other, "Invalid DOS signature"));
    }
    file.seek(SeekFrom::Start((dos_header.e_lfanew  + 4) as u64))?;
    let mut file_header: IMAGE_FILE_HEADER = std::mem::zeroed();
    file.read_exact(slice::from_raw_parts_mut(&mut file_header as *mut _ as *mut u8, std::mem::size_of::<IMAGE_FILE_HEADER>()))?;
    if file_header.Machine == 0x8664 {
        let mut nt_header_64: IMAGE_NT_HEADERS64 = std::mem::zeroed();
        file.seek(SeekFrom::Start(dos_header.e_lfanew as u64))?;
        file.read_exact(slice::from_raw_parts_mut(&mut nt_header_64 as *mut _ as *mut u8, std::mem::size_of::<IMAGE_NT_HEADERS64>()))?;
        NT_HEADER = Some(NtHeaders::Headers64(nt_header_64));
        crate::symbol::IMAGE_BASE = nt_header_64.OptionalHeader.ImageBase;
        section::parse_section(NtHeaders::Headers64(nt_header_64), &mut file)?;
        function::parse_pdata(nt_header_64.OptionalHeader.DataDirectory[3]);
    }
    else if file_header.Machine == 0x14c {
        let mut nt_header_32: IMAGE_NT_HEADERS32 = std::mem::zeroed();
        file.seek(SeekFrom::Start(dos_header.e_lfanew as u64))?;
        file.read_exact(slice::from_raw_parts_mut(&mut nt_header_32 as *mut _ as *mut u8, std::mem::size_of::<IMAGE_NT_HEADERS64>()))?;
        NT_HEADER = Some(NtHeaders::Headers32(nt_header_32));
        crate::symbol::IMAGE_BASE = nt_header_32.OptionalHeader.ImageBase as u64;
        section::parse_section(NtHeaders::Headers32(nt_header_32), &mut file)?;
        function::parse_pdata(nt_header_32.OptionalHeader.DataDirectory[3]);
    } else {
        return Err(io::Error::new(io::ErrorKind::Other, "only x64 - x32 file is supported"));
    }
    Ok(())
}
