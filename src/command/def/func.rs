use crate::usage::USAGE_DEF_FUNC;
use std::{io, ptr};
use std::io::Write;
use keystone_engine::{Arch, Keystone, Mode, OptionType, OptionValue};
use crate::ALL_ELM;
use crate::pefile::{NtHeaders, NT_HEADER};
use crate::symbol::SYMBOLS_V;
use crate::utils::*;

#[derive(Default, Debug)]
pub struct CrtFunc {
    pub code_str: Vec<String>,
    pub name: String,
    pub addr: u64
}




fn verify_insn(engine: *const Keystone, asm_line: &str) -> Result<(), String>{
    let mut new_line = asm_line.to_string();
    if let Some(sym) = unsafe {SYMBOLS_V.symbol_file.iter().find(|s|asm_line.contains(&s.name) && s.name.len() > 3)} {
        new_line = new_line.replace(&sym.name, "0x124");
    }
    unsafe {
        match (*engine).asm(new_line, 0) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("{e}"))
        }
    }
}


pub fn crt_func(arg: &[&str]) {
    if arg.len() < 2 {
        println!("{USAGE_DEF_FUNC}");
        return;
    }
    let mod_asm = {
        if let Some(nt_h) = unsafe {NT_HEADER} {
            match nt_h {
                NtHeaders::Headers32(_) =>  Mode::MODE_32,
                NtHeaders::Headers64(_) => Mode::MODE_64,
            }
        }else {
            eprintln!("{WAR_COLOR}the basic architecture is x64, if this function is intended to be on 32bits write 32bits code{RESET_COLOR}");
            Mode::MODE_64
        }
    };
    println!("{}:", arg[1]);
    let engine = Keystone::new(Arch::X86, mod_asm).unwrap();
    engine.option(OptionType::SYNTAX, OptionValue::SYNTAX_INTEL).unwrap();
    let mut vec_line = Vec::new();
    loop {
        print!("    ");
        let mut input = String::new();
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim().to_string();
        if input == ":q" {
            print!("\x1B[1A\x1B[2K");
            break
        }
        if input.contains("\n") {
            for line in input.lines() {
                match verify_insn(ptr::addr_of!(engine), &input) {
                    Ok(()) => vec_line.push(line.to_string()),
                    Err(e) => {
                        print!("\x1B[1A\x1B[2K");
                        println!("{ERR_COLOR}    {line} ; {e}{RESET_COLOR}");
                    },
                }
            }
        }else {
            match verify_insn(ptr::addr_of!(engine), &input) {
                Ok(()) => vec_line.push(input),
                Err(e) => {
                    print!("\x1B[1A\x1B[2K");
                    println!("{ERR_COLOR}    {input} ; {e}{RESET_COLOR}");
                },
            }
        }
    }
    unsafe {ALL_ELM.crt_func.push(CrtFunc{name: arg[1].to_string(), code_str: vec_line, addr: 0} );}
}