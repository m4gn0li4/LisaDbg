use std::io;
use iced_x86::{Decoder, DecoderOptions, Instruction};
use winapi::shared::minwindef::LPVOID;
use winapi::shared::ntdef::HANDLE;
use winapi::um::memoryapi::{ReadProcessMemory, VirtualProtectEx};
use winapi::um::winnt::{PAGE_EXECUTE_READWRITE, WOW64_CONTEXT};
use crate::command::breakpoint::get_addr_br;
use crate::dbg::BASE_ADDR;
use crate::dbg::dbg_cmd::mode_32::info_reg::ToValue32;
use crate::dbg::dbg_cmd::usages;
use crate::log::*;
use crate::pefile::function::FUNC_INFO;
use crate::symbol;

pub fn handle_disasm32(linev: &[&str], process_handle: HANDLE, ctx: WOW64_CONTEXT) {
    if linev.len() != 3 {
        println!("{}", usages::USAGE_DISASM);
        return;
    }

    let addr_str = linev[1];
    let count_str = linev[2];

    let addr = match get_addr_br(addr_str) {
        Ok(value) => value as u32 + unsafe { BASE_ADDR } as u32,
        Err(e) => {
            if e.contains("invalid target") {
                let addr = ctx.str_to_ctx(addr_str);
                if addr == 0 {
                    eprintln!("{}Invalid target: '{}'{}", ERR_COLOR, addr_str, RESET_COLOR);
                    return;
                } else {
                    addr
                }
            }else {
                eprintln!("{e}");
                return;
            }
        }
    };

    let count = match str_to::<usize>(count_str) {
        Ok(count) => count,
        Err(e) => {
            println!("{}Invalid count: {}{}", ERR_COLOR, e, RESET_COLOR);
            return;
        }
    };
    let mut size = 0;
    if let Some(func) = unsafe { FUNC_INFO .iter().find(|f|f.BeginAddress + BASE_ADDR as u32 <= addr && f.EndAddress + BASE_ADDR as u32 >= addr)} {
        size = (func.EndAddress - func.BeginAddress) as usize;
    }
    let mut buffer = vec![0u8; if count != usize::MAX {2093} else {size}];
    let mut old_protect: u32 = 0;
    unsafe {
        if VirtualProtectEx(process_handle, addr as LPVOID, buffer.len(), PAGE_EXECUTE_READWRITE, &mut old_protect) == 0 {
            eprintln!("{ERR_COLOR}Failed to remove memory protection at address {:#x}: {}{RESET_COLOR}", addr, io::Error::last_os_error());
            return;
        }
        if ReadProcessMemory(process_handle, addr as LPVOID, buffer.as_mut_ptr() as LPVOID, buffer.len(), std::ptr::null_mut()) == 0 {
            eprintln!("{ERR_COLOR}Failed to read process memory: {}{RESET_COLOR}", io::Error::last_os_error());
            return;
        }
        if VirtualProtectEx(process_handle, addr as LPVOID, buffer.len(), old_protect, &mut old_protect) == 0 {
            eprintln!("{ERR_COLOR}Failed to restore memory protection at address {:#x}: {}{}", addr, io::Error::last_os_error(), RESET_COLOR);
            return;
        }
    }
    let addr = addr as u64;
    let mut decoder = Decoder::with_ip(64, &buffer, addr, DecoderOptions::NONE);
    let mut instruction = Instruction::default();
    let mut i = 0;
    println!("\x1b[1m{: <16} {: <48} {: <32}{RESET_COLOR}", "Address", "Bytes", "Instruction");
    while decoder.can_decode() && i < count {
        decoder.decode_out(&mut instruction);
        let start_index = (instruction.ip() - addr) as usize;
        let instr_bytes = &buffer[start_index..start_index + instruction.len()];
        let byte_str = instr_bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join(" ");

        if let Some(sym) = unsafe { symbol::SYMBOLS_V.symbol_file.iter().find(|s| s.offset + BASE_ADDR as i64 == instruction.ip() as i64) } {
            println!("{ADDR_COLOR}{:#x}:", instruction.ip());
            println!("{:#x}:                                        {SYM_COLOR}{}:{RESET_COLOR}", instruction.ip(), sym.name);
        }

        let mut instructions = instruction.to_string();
        if instruction.is_ip_rel_memory_operand() {
            if let Some(sym) = unsafe { symbol::SYMBOLS_V.symbol_file.iter().find(|s| s.offset + BASE_ADDR as i64 == instruction.ip_rel_memory_address() as i64) } {
                instructions = instructions.replace(&format!("{:X}h", instruction.ip_rel_memory_address()), &format!("{SYM_COLOR}{}{INSTR_COLOR}", sym.name));
            }
        }
        if instruction.near_branch_target() != 0 {
            if let Some(sym) = unsafe { symbol::SYMBOLS_V.symbol_file.iter().find(|s| s.offset + BASE_ADDR as i64 == instruction.ip_rel_memory_address() as i64) } {
                instructions = instructions.replace(&format!("{:016X}h", instruction.near_branch_target()), &sym.name)
            }
        }
        println!("{ADDR_COLOR}{:#x}: {BYTES_COLOR}{:<48} {INSTR_COLOR}{instructions}{RESET_COLOR}", instruction.ip(), byte_str);
        i += 1;
    }
}