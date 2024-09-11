use std::{io, slice};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use winapi::um::winnt::IMAGE_SECTION_HEADER;
use crate::pefile::{get_name, NtHeaders, Section};

pub static mut SECTION_VS: Vec<Section> = Vec::new();

unsafe fn read_section_headers(file: &mut File, number_of_sections: usize) -> Result<Vec<IMAGE_SECTION_HEADER>, io::Error> {
    let mut section_h = Vec::with_capacity(number_of_sections);
    for _ in 0..number_of_sections {
        let mut section_header: IMAGE_SECTION_HEADER = std::mem::zeroed();
        file.read_exact(slice::from_raw_parts_mut(&mut section_header as *mut _ as *mut u8, std::mem::size_of_val(&section_header)))?;
        section_h.push(section_header);
    }
    Ok(section_h)
}

unsafe fn process_section(file: &mut File, section: IMAGE_SECTION_HEADER, smptroffs: u64) -> Result<(), io::Error> {
    let name_bytes = get_name(smptroffs, file, section.Name.to_vec());
    let name = String::from_utf8_lossy(&name_bytes).trim_end_matches(char::from(0)).to_string();
    let mut content = vec![0u8; section.SizeOfRawData as usize];
    file.seek(SeekFrom::Start(section.PointerToRawData as u64))?;
    file.read_exact(&mut content)?;
    SECTION_VS.push(Section { name, content, addr: section.VirtualAddress , ptr2raw: section.PointerToRawData});
    Ok(())
}



pub unsafe fn parse_section(nt_headers: NtHeaders, file: &mut File) -> Result<(), io::Error> {
    match nt_headers {
        NtHeaders::Headers32(nt_headers_32) => {
            let num_sec = nt_headers_32.FileHeader.NumberOfSections as usize;
            let smptroffs = nt_headers_32.FileHeader.PointerToSymbolTable as u64 + (nt_headers_32.FileHeader.NumberOfSymbols * 18) as u64;
            for section in read_section_headers(file, num_sec)? {
                process_section(file, section, smptroffs)?;
            }
            Ok(())
        }
        NtHeaders::Headers64(nt_headers_64) => {
            let num_sec = nt_headers_64.FileHeader.NumberOfSections as usize;
            let smptroffs = nt_headers_64.FileHeader.PointerToSymbolTable as u64 + (nt_headers_64.FileHeader.NumberOfSymbols * 18) as u64;
            for section in read_section_headers(file, num_sec)? {
                process_section(file, section, smptroffs)?;
            }
            Ok(())
        }
    }
}
