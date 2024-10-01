use crate::pefile::{get_name, NtHeaders, Section};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::{io, ptr, slice};
use winapi::um::winnt::IMAGE_SECTION_HEADER;

pub static mut SECTION_VS: Vec<Section> = Vec::new();

unsafe fn read_section_headers(file: &mut File, num_sec: usize) -> Result<Vec<IMAGE_SECTION_HEADER>, io::Error> {
    let mut section_h = Vec::with_capacity(num_sec);
    for _ in 0..num_sec {
        let mut section_header: IMAGE_SECTION_HEADER = std::mem::zeroed();
        file.read_exact(slice::from_raw_parts_mut(ptr::addr_of_mut!(section_header) as *mut u8, size_of::<IMAGE_SECTION_HEADER>()))?;
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
    SECTION_VS.push(Section { name, content, addr: section.VirtualAddress});
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
