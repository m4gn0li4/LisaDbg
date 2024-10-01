use crate::symbol::{SymbolFile, SYMBOLS_V};
use crate::utils::{ERR_COLOR, RESET_COLOR};
use std::ffi::{c_char, CStr};
use std::{io, mem, ptr};
use winapi::shared::minwindef::{LPVOID, TRUE};
use winapi::shared::ntdef::HANDLE;
use winapi::um::dbghelp::{AddrModeFlat, StackWalk64, SymInitializeW, STACKFRAME64};
use winapi::um::libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryA};
use winapi::um::winnt::{CONTEXT, WOW64_CONTEXT};

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
            return Some(*frame);
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

pub unsafe fn get_frame_st(h_proc: HANDLE, h_thread: HANDLE, ctx: CONTEXT) {
    let mut ctx = ctx;
    stack_walk(0x8664, h_proc, h_thread, ptr::addr_of_mut!(ctx) as LPVOID, ctx.Rip, ctx.Rsp, ctx.Rbp);
}

fn stack_walk(machine: u32, h_proc: HANDLE, h_thread: HANDLE, pctx: LPVOID, rip: u64, rsp: u64, rbp: u64) {
    let mut stack_frame: STACKFRAME64 = unsafe { mem::zeroed() };
    stack_frame.AddrPC.Offset = rip;
    stack_frame.AddrPC.Mode = AddrModeFlat;
    stack_frame.AddrStack.Offset = rsp;
    stack_frame.AddrStack.Mode = AddrModeFlat;
    stack_frame.AddrFrame.Offset = rbp;
    stack_frame.AddrFrame.Mode = AddrModeFlat;
    unsafe {
        if SymInitializeW(h_proc, ptr::null(), TRUE) == 0 {
            eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to initialize symbols. Error: {}", io::Error::last_os_error());
            return;
        }
        while StackWalk64(machine, h_proc, h_thread, &mut stack_frame, pctx, None,
                          Some(winapi::um::dbghelp::SymFunctionTableAccess64), Some(winapi::um::dbghelp::SymGetModuleBase64), None) != 0 {
            ST_FRAME.push(stack_frame);
        }
    }
}

pub unsafe fn get_frame_st32(h_proc: HANDLE, h_thread: HANDLE, ctx: WOW64_CONTEXT) {
    let mut ctx = ctx;
    stack_walk(0x14c, h_proc, h_thread, ptr::addr_of_mut!(ctx) as LPVOID, ctx.Eip as u64, ctx.Esp as u64, ctx.Ebp as u64);
}

pub unsafe fn get_local_sym(h_proc: HANDLE, addr_sym: u64) {
    let symbol_pe = LoadLibraryA("symbol_pe.dll\0".as_ptr() as *const i8);
    if symbol_pe.is_null() {
        panic!("failed to load symbol_pe.dll");
    }
    let get_local_var: unsafe extern "C" fn(HANDLE, u64, &mut usize) -> *mut LocalSym = mem::transmute(GetProcAddress(symbol_pe, "GetLocalVar\0".as_ptr() as *const i8));
    let mut len = 0;
    let sym = get_local_var(h_proc, addr_sym, &mut len);
    if sym.is_null() {
        return;
    }
    LEN = len;
    let get_tag_str: unsafe extern "C" fn(u32) -> *const c_char = mem::transmute(GetProcAddress(symbol_pe, "GetTagString\0".as_ptr() as *const i8));
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
    let free_sym: unsafe extern "C" fn(*mut LocalSym, usize) = mem::transmute(GetProcAddress(symbol_pe, "freeLocalSym\0".as_ptr() as *const i8));
    free_sym(sym, len);
    FreeLibrary(symbol_pe);
}
