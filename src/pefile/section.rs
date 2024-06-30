use std::{io, slice};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use winapi::um::winnt::{IMAGE_NT_HEADERS, IMAGE_SECTION_HEADER};
use crate::pefile::{get_name, Section};


pub static mut SECTION_VS: Vec<Section> = Vec::new();

pub unsafe fn parse_section(nt_headers: IMAGE_NT_HEADERS, file: &mut File) -> Result<(), io::Error>{
    if nt_headers.FileHeader.Machine == 0x8664 {
        let number_of_sections = nt_headers.FileHeader.NumberOfSections as usize;
        let smptroffs = nt_headers.FileHeader.PointerToSymbolTable as u64 + (nt_headers.FileHeader.NumberOfSymbols * 18) as u64;
        let mut section_h = Vec::with_capacity(number_of_sections);
        for _ in 0..number_of_sections {
            let mut section_header: IMAGE_SECTION_HEADER = std::mem::zeroed();
            file.read_exact(slice::from_raw_parts_mut(&mut section_header as *mut _ as *mut u8, std::mem::size_of_val(&section_header)))?;
            section_h.push(section_header);
        }
        for section in section_h {
            let name_bytes = get_name(smptroffs, file, section.Name.to_vec());
            let name = String::from_utf8_lossy(&name_bytes).trim_end_matches(char::from(0)).to_string();
            let mut content = vec![0u8; section.SizeOfRawData as usize];
            file.seek(SeekFrom::Start(section.PointerToRawData as u64))?;
            file.read_exact(&mut content)?;
            SECTION_VS.push(Section { name, content, addr: section.VirtualAddress as usize })
        }
        Ok(())
    }else {
        Err(io::Error::new(io::ErrorKind::Other, "only x64 file is supported"))
    }
}