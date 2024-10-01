use crate::dbg::dbg_cmd::usages;
use crate::dbg::BASE_ADDR;
use crate::pefile::function::FUNC_INFO;
use crate::pefile::NT_HEADER;
use crate::symbol::{SymbolFile, SYMBOLS_V};
use crate::utils::*;
use crate::utils::{str_to, ERR_COLOR, RESET_COLOR};
use iced_x86::{Decoder, DecoderOptions, Instruction, Mnemonic};
use std::{io, mem, ptr};
use winapi::shared::minwindef::LPVOID;
use winapi::shared::ntdef::HANDLE;
use winapi::um::memoryapi::{ReadProcessMemory, VirtualProtectEx, VirtualQueryEx};
use winapi::um::winnt::{
    CONTEXT, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READWRITE, PAGE_READONLY, WOW64_CONTEXT,
};

pub struct JAddr {
    addr: u64,
    num_target: usize,
}

fn finder(sym: &SymbolFile, target: u64) -> bool {
    if sym.offset < 0 {
        false
    } else {
        sym.offset as u64 + unsafe { BASE_ADDR } == target
    }
}

pub fn handle_disasm(linev: &[&str], h_proc: HANDLE, ctx: CONTEXT) {
    if linev.len() < 2 {
        println!("{}", usages::USAGE_DISASM);
        return;
    }

    let addr_str = linev[1];
    let count_str = linev.get(2);

    let addr = match get_addr_va(addr_str, ctx) {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };
    disasm(h_proc, addr, count_str);
}

pub fn handle_disasm32(linev: &[&str], h_proc: HANDLE, ctx: WOW64_CONTEXT) {
    if linev.len() < 2 {
        println!("{}", usages::USAGE_DISASM);
        return;
    }

    let addr_str = linev[1];
    let count_str = linev.get(2);

    let addr = match get_addr_va32(addr_str, ctx) {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("{e}");
            return;
        }
    };
    disasm(h_proc, addr as u64, count_str);
}

fn disasm(h_proc: HANDLE, addr: u64, count_str: Option<&&str>) {
    let count = if count_str.is_some() {
        match str_to::<usize>(count_str.unwrap()) {
            Ok(count) => count,
            Err(e) => {
                println!("{}Invalid count: {}{}", ERR_COLOR, e, RESET_COLOR);
                return;
            }
        }
    } else {
        usize::MAX
    };
    let size;
    unsafe {
        if FUNC_INFO.len() != 0 {
            if let Some(func) = FUNC_INFO.iter().find(|f| {
                f.BeginAddress as u64 + BASE_ADDR <= addr && f.EndAddress as u64 + BASE_ADDR > addr
            }) {
                size = (func.EndAddress as u64 + BASE_ADDR - addr) as usize;
            } else {
                size = 2093;
            }
        } else {
            let mut mem_info = mem::zeroed();
            if VirtualQueryEx(h_proc, addr as LPVOID, &mut mem_info, size_of::<MEMORY_BASIC_INFORMATION>()) == 0 {
                eprintln!("{ERR_COLOR}Failed to get the size of region of {:#x} : {}{RESET_COLOR}", addr, io::Error::last_os_error());
                return;
            }
            size = mem_info.RegionSize as usize;
        }
    }

    let mut buffer = vec![0u8; size];
    let mut old_protect: u32 = 0;
    unsafe {
        if VirtualProtectEx(h_proc, addr as LPVOID, buffer.len(), PAGE_EXECUTE_READWRITE, &mut old_protect) == 0
        {
            eprintln!("{ERR_COLOR}Failed to remove memory protection at address {:#x}: {}{RESET_COLOR}", addr, io::Error::last_os_error());
            return;
        }
        if ReadProcessMemory(h_proc, addr as LPVOID, buffer.as_mut_ptr() as LPVOID, buffer.len(), ptr::null_mut()) == 0 {
            eprintln!(
                "{ERR_COLOR}Failed to read process memory: {}{RESET_COLOR}",
                io::Error::last_os_error()
            );
            return;
        }
        if VirtualProtectEx(h_proc, addr as LPVOID, buffer.len(), old_protect, &mut old_protect) == 0 {
            eprintln!("{ERR_COLOR}Failed to restore memory protection at address {:#x}: {}{RESET_COLOR}", addr, io::Error::last_os_error());
            return;
        }
    }
    let mut decoder = Decoder::with_ip(unsafe { NT_HEADER }.unwrap().get_bitness() as u32, &buffer, addr, DecoderOptions::NONE);
    let mut insn = Instruction::default();
    let mut i = 0;
    let j_jump = first_it(Decoder::with_ip(unsafe { NT_HEADER }.unwrap().get_bitness() as u32, &buffer, addr, DecoderOptions::NONE), count);
    let mut last_insn_is_ret = false;
    println!(
        "\x1b[1m{: <16} {: <48} {: <32}{RESET_COLOR}",
        "Address", "Bytes", "Instruction"
    );

    while decoder.can_decode() && i < count {
        decoder.decode_out(&mut insn);
        let start_index = (insn.ip() - addr) as usize;
        let instr_bytes = &buffer[start_index..start_index + insn.len()];
        let byte_str = instr_bytes.iter().map(|b| format!("{:02X}", b)).collect::<Vec<String>>().join(" ");
        let mut insns = insn.to_string();
        if last_insn_is_ret || j_jump.iter().any(|j| j.addr == insn.ip()) {
            println!("{ADDR_COLOR}{:#x}: ", insn.ip());
            println!("{ADDR_COLOR}{:#x}: ", insn.ip());
            println!("{ADDR_COLOR}{:#x}: {:<40}{}{RESET_COLOR}", insn.ip(), "", format!("label_{:x}:", insn.ip()));
            last_insn_is_ret = false;
        }
        if insn.near_branch_target() != 0 {
            unsafe {
                if let Some(sym) = SYMBOLS_V.symbol_file.iter().find(|&s| finder(s, insn.near_branch_target())) {
                    insns = insns.replace(&format!("{:016X}h", insn.near_branch_target()), &format!("{MAGENTA}{}{RESET_COLOR}", sym.name))
                } else {
                    insns = insns.replace(&format!("{:016X}h", insn.near_branch_target()), &format!("{MAGENTA}label_{:x}{RESET_COLOR}", insn.near_branch_target()));
                }
            }
        }

        if insn.is_ip_rel_memory_operand() {
            unsafe {
                if let Some(sym) = SYMBOLS_V.symbol_file.iter().find(|&s| finder(s, insn.ip_rel_memory_address())) {
                    insns = insns.replace(&format!("{:X}h", insn.ip_rel_memory_address()), &format!("{MAGENTA}{}{INSTR_COLOR}", sym.name))
                }
                let mut mem_info = mem::zeroed();
                if VirtualQueryEx(h_proc, insn.ip_rel_memory_address() as LPVOID, &mut mem_info, size_of::<MEMORY_BASIC_INFORMATION>()) == 0 {
                    eprintln!("{ADDR_COLOR}{:#x}:    {ERR_COLOR}Failed to get memory info of {:#x} : {}{RESET_COLOR}", insn.ip(), insn.ip_rel_memory_address(), io::Error::last_os_error());
                } else {
                    if mem_info.Protect == PAGE_READONLY {
                        let deref_size = insn.memory_size().size();
                        if deref_size != 0 {
                            let mut res = 0u64;
                            if ReadProcessMemory(h_proc, insn.ip_rel_memory_address() as LPVOID, ptr::addr_of_mut!(res) as LPVOID, deref_size, &mut 0) == 0 {
                                eprintln!("{ADDR_COLOR}{:#x}:    {ERR_COLOR}Failed to read memory at {:#x} : {}{RESET_COLOR}", insn.ip(), insn.ip_rel_memory_address(), io::Error::last_os_error());
                            } else {
                                insns.push_str(&format!("{CYAN_COLOR} ; {:#x}", res));
                            }
                        } else {
                            let mut buffer = vec![0u8; 260];
                            if ReadProcessMemory(h_proc, insn.ip_rel_memory_address() as LPVOID, buffer.as_mut_ptr() as LPVOID, 260, &mut 0) == 0 {
                                eprintln!("{ADDR_COLOR}{:#x}:    {ERR_COLOR}Failed to read memory at {:#x} : {}{RESET_COLOR}", insn.ip(), insn.ip_rel_memory_address(), io::Error::last_os_error());
                            } else {
                                insns.push_str(&format!(" ; {RESET_COLOR}\"{}\"", crate::dbg::memory::deref_mem::espc(&buffer[..buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len())])));
                            }
                        }
                    }
                }
            }
        }
        if insn.immediate64() != 0 {
            unsafe {
                if let Some(sym) = SYMBOLS_V.symbol_file.iter().find(|&s| finder(s, insn.immediate64())) {
                    insns = insns.replace(&format!("{:X}h", insn.immediate64()), &sym.name);
                }
            }
        }
        println!("{ADDR_COLOR}{:#x}: {BYTES_COLOR}{:<48} {VALUE_COLOR}{insns}{RESET_COLOR}", insn.ip(), byte_str);
        if insn.mnemonic() == Mnemonic::Ret {
            last_insn_is_ret = true;
        }
        i += 1;
    }
}

pub fn first_it(decoder: Decoder, count: usize) -> Vec<JAddr> {
    let mut decoder = decoder;
    let mut insn = Instruction::default();
    let mut i = 0;
    let mut result: Vec<JAddr> = Vec::new();
    while decoder.can_decode() && i < count {
        decoder.decode_out(&mut insn);
        if insn.near_branch_target() != 0 {
            if let Some(jmp) = result.iter_mut().find(|f| f.addr == insn.near_branch_target()) {
                jmp.num_target += 1
            } else {
                result.push(JAddr {
                    addr: insn.near_branch_target(),
                    num_target: 0,
                });
            }
        }
        i += 1
    }
    result
}
