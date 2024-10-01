use crate::utils::*;
use crate::{pefile, symbol, ALL_ELM};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn handle_change_file(linev: &[&str], line: &str) {
    if linev.len() > 1 {
        let file_str = line[4..].replace("\"", "");
        let file_str = file_str.trim_start().trim_end();
        if Path::new(file_str).exists() {
            unsafe {
                let mut file = File::open(file_str).unwrap();
                let mut mz_head = [0u8; 2];
                file.read_exact(&mut mz_head).unwrap();
                if mz_head == *b"MZ" {
                    ALL_ELM.file = Some(file_str.to_string());
                    symbol::SYMBOLS_V.symbol_file.clear();
                    if let Err(e) = pefile::parse_header() {
                        eprintln!("{ERR_COLOR}{e}{RESET_COLOR}");
                        return;
                    }
                    println!("{VALID_COLOR}Now the file context is '{}'{RESET_COLOR}", ALL_ELM.file.clone().unwrap());
                    symbol::load_symbol();
                } else {
                    eprintln!("{ERR_COLOR}please specify a valid pe file{RESET_COLOR}");
                }
            }
        } else {
            eprintln!("{ERR_COLOR}the path {} doesn't exist{RESET_COLOR}", file_str);
        }
    }else {
        println!("{VALID_COLOR}USAGE: file <path>{RESET_COLOR}");
    }
}
