mod dbg_cmd;
mod memory;
mod handle_breakpoint;
pub mod hook;

use winapi::um::fileapi::GetFinalPathNameByHandleA;
use crate::log::*;
use winapi::um::winbase::{DEBUG_PROCESS, INFINITE};
use winapi::um::handleapi::CloseHandle;
use std::{io, ptr};
use std::ffi::CString;
use std::mem;
use std::os::raw::c_char;
use winapi::um::winnt::*;
use winapi::um::processthreadsapi::*;
use winapi::um::minwinbase::*;
use winapi::um::debugapi::*;
use winapi::shared::minwindef::{FALSE, DWORD, LPVOID};
use crate::{OPTION, ste};
use crate::dbg::hook::HOOK_FUNC;
use crate::pefile::function::CR_FUNCTION;
use crate::ste::ST_OVER_ADDR;

static mut INSNBRPT: Vec<InsnInBrpt> = Vec::new();
struct InsnInBrpt {
    addr: u64,
    last_oc: u8,
}


#[derive(Debug, Default)]
pub struct StBeginFunc {
    pub sp: u64,
    pub ret_addr: u64,
    begin_func_addr: u64,
}

pub static mut BASE_ADDR: u64 = 0;

pub static mut ST_BEGIN_INFO: StBeginFunc = StBeginFunc { sp: 0, ret_addr: 0, begin_func_addr: 0};





fn debug_loop(process_handle: HANDLE) {
    unsafe {
        let mut debug_event = mem::zeroed::<DEBUG_EVENT>();
        let mut continue_dbg = true;
        let mut main_module_base: LPVOID;
        let mut single_step = false;
        while continue_dbg {
            WaitForDebugEvent(&mut debug_event, INFINITE);
            match debug_event.dwDebugEventCode {
                EXCEPTION_DEBUG_EVENT => {
                    if debug_event.u.Exception().ExceptionRecord.ExceptionCode == EXCEPTION_BREAKPOINT {
                        let breakpoint_addr = debug_event.u.Exception().ExceptionRecord.ExceptionAddress as u64;
                        if breakpoint_addr >= BASE_ADDR && ste::STE_RETURN_ADDR.contains(&(breakpoint_addr - BASE_ADDR)) {
                            handle_breakpoint::handle_stret(process_handle, debug_event, breakpoint_addr, &mut continue_dbg, &mut single_step)
                        }
                        else if let Some(hook_func) = HOOK_FUNC.iter().find(|a|a.target == breakpoint_addr - BASE_ADDR) {
                            handle_breakpoint::handle_hook_func(process_handle, *hook_func, debug_event, &mut continue_dbg);
                        }
                        else if breakpoint_addr > BASE_ADDR && OPTION.breakpoint_addr.contains(&(breakpoint_addr - BASE_ADDR)) {
                            handle_breakpoint::handle_br(process_handle, debug_event, breakpoint_addr, &mut continue_dbg, &mut single_step)
                        }
                    }

                    else if single_step && debug_event.u.Exception().ExceptionRecord.ExceptionCode == EXCEPTION_SINGLE_STEP {
                        single_step = false;
                        let h_thread = OpenThread(THREAD_GET_CONTEXT | THREAD_SET_CONTEXT, FALSE, debug_event.dwThreadId);
                        if !h_thread.is_null() {
                            let mut ctx = mem::zeroed::<CONTEXT>();
                            ctx.ContextFlags = CONTEXT_FULL;
                            if GetThreadContext(h_thread, &mut ctx) != 0 {
                                for insn in &*INSNBRPT {
                                    if insn.addr == ctx.Rip {
                                        memory::set_breakpoint(process_handle, BASE_ADDR as LPVOID, insn.addr - BASE_ADDR)
                                    }
                                }
                            } else {
                                eprintln!("\n[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to get thread context: {}", io::Error::last_os_error());
                            }
                            CloseHandle(h_thread);
                        }
                    }
                }
                CREATE_PROCESS_DEBUG_EVENT => {
                    println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Process created at address: {:x?}", debug_event.u.CreateProcessInfo().lpBaseOfImage);
                    main_module_base = debug_event.u.CreateProcessInfo().lpBaseOfImage;
                    BASE_ADDR = main_module_base as u64;
                    for addr in &OPTION.breakpoint_addr {
                        memory::set_breakpoint(process_handle, main_module_base, *addr);
                    }
                    for addr_over in ST_OVER_ADDR.clone() {
                        memory::set_addr_over(process_handle, addr_over);
                    }
                    for crt_func in CR_FUNCTION.iter_mut() {
                        memory::set_cr_function(process_handle, crt_func);
                    }
                }
                EXIT_PROCESS_DEBUG_EVENT => {
                    println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Process exited");
                    continue_dbg = false;
                }
                CREATE_THREAD_DEBUG_EVENT => println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Thread created"),
                EXIT_THREAD_DEBUG_EVENT => println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Thread exited"),
                LOAD_DLL_DEBUG_EVENT => {
                    let dll_base = debug_event.u.LoadDll().lpBaseOfDll;
                    let h_file = debug_event.u.LoadDll().hFile;
                    let mut buffer: [c_char; winapi::shared::minwindef::MAX_PATH] = [0; winapi::shared::minwindef::MAX_PATH];
                    let len = GetFinalPathNameByHandleA(h_file, buffer.as_mut_ptr(), winapi::shared::minwindef::MAX_PATH as DWORD, 0);
                    if len > 0 {
                        let path = std::slice::from_raw_parts(buffer.as_ptr() as *const u8, len as usize);
                        if let Ok(cstr) = std::str::from_utf8(path) {
                            let display_path = if cstr.starts_with(r"\\?\") {
                                &cstr[4..]
                            } else {
                                cstr
                            };
                            println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Dll at address: {:x?} has been loaded ;{}", dll_base, display_path);
                        } else {
                            println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Dll at address: {:x?} has been loaded", dll_base);
                        }
                    } else {
                        println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Dll at address: {:x?} has been loaded", dll_base);
                    }
                }
                UNLOAD_DLL_DEBUG_EVENT =>  println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Dll at address {:x?} has been unloaded", debug_event.u.UnloadDll().lpBaseOfDll),
                OUTPUT_DEBUG_STRING_EVENT => println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Debug string output"),
                _ => {}
            }
            ContinueDebugEvent(debug_event.dwProcessId, debug_event.dwThreadId, DBG_CONTINUE);
        }
    }
}




fn start_debugging(exe_path: &str) -> bool {
    unsafe {
        let mut si = mem::zeroed::<STARTUPINFOA>();
        let mut pi = mem::zeroed::<PROCESS_INFORMATION>();
        si.cb = mem::size_of::<STARTUPINFOA>() as u32;
        let exe_path_cstr = CString::new(exe_path).unwrap();
        if CreateProcessA(ptr::null_mut(), exe_path_cstr.as_ptr() as *mut i8, ptr::null_mut(), ptr::null_mut(), false as i32, DEBUG_PROCESS,
                          ptr::null_mut(), ptr::null_mut(), &mut si, &mut pi) == 0
        {
            eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> CreateProcess failed : {}", io::Error::last_os_error());
            return false;
        }

        debug_loop(pi.hProcess);
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);
    }
    true
}


pub fn run() {
    if unsafe { OPTION.file.is_some() } {
        let arg = unsafe {
            if OPTION.arg.is_some() {
                format!("{} {}", &OPTION.file.clone().unwrap(), &OPTION.arg.clone().unwrap())
            }else {
                OPTION.file.clone().unwrap()
            }
        };
        if !start_debugging(&arg) {
            eprintln!("{ERR_COLOR}Failed to start debugging{RESET_COLOR}");
        }
    } else {
        eprintln!("{ERR_COLOR}Please enter a file path{RESET_COLOR}");
    }
}
