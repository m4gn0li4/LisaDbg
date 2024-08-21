use std::{io, mem, ptr};
use std::ffi::{c_char, CStr};
use libloading::Symbol;
use winapi::shared::minwindef::{LPVOID, TRUE};
use winapi::shared::ntdef::{HANDLE, PVOID};
use winapi::um::dbghelp::{AddrModeFlat, STACKFRAME64, StackWalk64, SymInitializeW};
use winapi::um::winnt::{CONTEXT, WOW64_CONTEXT};
use crate::utils::{ERR_COLOR, RESET_COLOR};
use crate::symbol::{SYMBOL_PE, SymbolFile, SYMBOLS_V};

pub static mut ST_FRAME: Vec<STACKFRAME64> = Vec::new();
pub static mut LEN: usize = 0;

#[repr(C)]
#[derive(Debug)]
pub struct LocalSym {
    pub size: u32,
    pub value: u64,
    pub address: u64,
    pub tag: u32,
    pub name: *const c_char,
    pub filename: *const c_char,
    pub line: u32,
    pub register: u32,
}



pub unsafe fn get_real_frame(rip: u64) -> Option<STACKFRAME64> {
    for frame in &*ST_FRAME {
        if frame.AddrPC.Offset == rip {
            return Some(*frame)
        }
    }
    None
}




pub unsafe fn get_frame_before_func(rip: u64) -> Option<STACKFRAME64> {
    for (i, frame) in ST_FRAME.iter().enumerate() {
        if frame.AddrPC.Offset == rip {
            return ST_FRAME.get(i + 1).cloned();
        }
    }
    None
}







pub unsafe fn get_frame_st(process_handle: HANDLE, h_thread: HANDLE, ctx: CONTEXT) {
    let mut ctx = ctx;
    let mut stack_frame: STACKFRAME64 = mem::zeroed();
    stack_frame.AddrPC.Offset = ctx.Rip;
    stack_frame.AddrPC.Mode = AddrModeFlat;
    stack_frame.AddrStack.Offset = ctx.Rsp;
    stack_frame.AddrStack.Mode = AddrModeFlat;
    stack_frame.AddrFrame.Offset = ctx.Rbp;
    stack_frame.AddrFrame.Mode = AddrModeFlat;
    if SymInitializeW(process_handle, ptr::null(), TRUE) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to initialize symbols. Error: {}", io::Error::last_os_error());
        return;
    }
    while StackWalk64(0x8664, process_handle, h_thread, &mut stack_frame, &mut ctx as *mut _ as PVOID, None,
                      Some(winapi::um::dbghelp::SymFunctionTableAccess64), Some(winapi::um::dbghelp::SymGetModuleBase64), None) != 0 {
        ST_FRAME.push(stack_frame)
    }
}





pub unsafe fn get_frame_st32(process_handle: HANDLE, h_thread: HANDLE, ctx: WOW64_CONTEXT){
    let mut ctx = ctx;
    let mut stack_frame: STACKFRAME64 = mem::zeroed();
    stack_frame.AddrPC.Offset = ctx.Eip as u64;
    stack_frame.AddrPC.Mode = AddrModeFlat;
    stack_frame.AddrStack.Offset = ctx.Esp as u64;
    stack_frame.AddrStack.Mode = AddrModeFlat;
    stack_frame.AddrFrame.Offset = ctx.Ebp as u64;
    stack_frame.AddrFrame.Mode = AddrModeFlat;
    if SymInitializeW(process_handle, ptr::null(), TRUE) == 0 {
        eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to initialize symbols. Error: {}", io::Error::last_os_error());
        return;
    }
    while StackWalk64(0x14c, process_handle, h_thread, &mut stack_frame, &mut ctx as *mut _ as LPVOID, None,
                      Some(winapi::um::dbghelp::SymFunctionTableAccess64), Some(winapi::um::dbghelp::SymGetModuleBase64), None) == TRUE{
        ST_FRAME.push(stack_frame);
    }
}





pub unsafe fn get_local_sym(process_handle: HANDLE, addr_sym: u64) {
    let get_local_var: Symbol<unsafe extern "C" fn(HANDLE, u64, &mut usize) -> *mut LocalSym> = SYMBOL_PE.get(b"GetLocalVar").expect("GetLocalVar symbol is not found in symbol_pe.dll");
    let mut len = 0;
    let sym = get_local_var(process_handle, addr_sym, &mut len);
    if sym.is_null() {
        return;
    }
    LEN = len;
    let get_tag_str: Symbol<unsafe extern "C" fn(u32) -> *const c_char> = SYMBOL_PE.get(b"GetTagString").unwrap();
    let sym_ar = std::slice::from_raw_parts(sym, len);
    for sym in sym_ar {
        let sym_file = SymbolFile {
            name: CStr::from_ptr(sym.name).to_string_lossy().to_string(),
            value_str: sym.value.to_string(),
            types_e: CStr::from_ptr(get_tag_str(sym.tag)).to_string_lossy().to_string() + " (local)",
            filename: "".to_string(),
            offset: sym.address as i32 as i64,
            size: sym.size as usize,
            line: sym.line as usize,
            register: sym.register,
        };
        SYMBOLS_V.symbol_file.push(sym_file);
    }
    let free_sym: Symbol<unsafe extern "C" fn(*mut LocalSym, usize)> = SYMBOL_PE.get(b"freeLocalSym").unwrap();
    free_sym(sym, len);
}