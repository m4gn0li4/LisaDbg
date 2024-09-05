use std::{io, mem, ptr};
use std::ffi::CString;
use std::os::raw::c_char;
use winapi::shared::minwindef::LPVOID;
use winapi::um::debugapi::{ContinueDebugEvent, DebugActiveProcessStop, WaitForDebugEventEx};
use winapi::um::fileapi::GetFinalPathNameByHandleA;
use winapi::um::handleapi::CloseHandle;
use winapi::um::memoryapi::{VirtualFreeEx, VirtualQueryEx};
use winapi::um::minwinbase::*;
use winapi::um::processthreadsapi::{CreateProcessA, PROCESS_INFORMATION, STARTUPINFOA};
use winapi::um::winbase::{DEBUG_PROCESS, INFINITE};
use winapi::um::winnt::*;
use crate::command::hook::HOOK_FUNC;
use crate::command::skip::SKIP_ADDR;
use crate::command::stret;
use crate::dbg::*;
use crate::OPTION;
use crate::pefile::function::CR_FUNCTION;

fn debug_loop(process_handle: HANDLE) {
    unsafe {
        let mut debug_event = mem::zeroed::<DEBUG_EVENT>();
        let mut continue_dbg = true;
        while continue_dbg {
            if WaitForDebugEventEx(&mut debug_event, INFINITE) == 0 {
                eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to WaitForDebugEventEx : {}", io::Error::last_os_error());
                stop_dbg(process_handle, debug_event);
            }
            match debug_event.dwDebugEventCode {
                EXCEPTION_DEBUG_EVENT => {
                    let except_addr = debug_event.u.Exception().ExceptionRecord.ExceptionAddress as u64;
                    match debug_event.u.Exception().ExceptionRecord.ExceptionCode {
                        EXCEPTION_BREAKPOINT | STATUS_WX86_BREAKPOINT => {
                            if let Some(hook_func) = HOOK_FUNC.iter().find(|a| a.target == except_addr - BASE_ADDR) {
                                handle_point::handle_hook_func(process_handle, *hook_func, debug_event, &mut continue_dbg);
                            } else if except_addr > BASE_ADDR && OPTION.breakpoint_addr.contains(&(except_addr - BASE_ADDR)) {
                                handle_point::handle_br(process_handle, debug_event, except_addr, &mut continue_dbg);
                            }
                        }
                        EXCEPTION_SINGLE_STEP | STATUS_WX86_SINGLE_STEP => handle_point::handle_watchpoint(debug_event, process_handle, &mut continue_dbg),
                        EXCEPTION_ARRAY_BOUNDS_EXCEEDED => {
                            eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> The code tries to access an invalid index in the table : {:#x}", debug_event.u.Exception().ExceptionRecord.ExceptionAddress as u64);
                            continue_dbg = false;
                        }
                        EXCEPTION_DATATYPE_MISALIGNMENT => {
                            eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> An alignment problem occurred at address {:#x} and the system does not provide alignment", except_addr);
                            continue_dbg = false;
                        }
                        EXCEPTION_FLT_DENORMAL_OPERAND => {
                            eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> One of the operands of a floating point operation is too small to be considered a floating point at address {:#x}", except_addr);
                            continue_dbg = false;
                        }
                        EXCEPTION_FLT_DIVIDE_BY_ZERO => {
                            eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> The thread attempted to divide a floating point value by a floating point divisor of zero at address {:#x}", except_addr);
                            continue_dbg = false;
                        }
                        EXCEPTION_FLT_INEXACT_RESULT => {
                            eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> The result of a floating point operation cannot be represented exactly as a decimal fraction at address {:#x}", except_addr);
                            continue_dbg = false;
                        }
                        EXCEPTION_FLT_INVALID_OPERATION => {
                            eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> An error with floating point numbers occurred at address {:#x}", except_addr);
                            continue_dbg = false;
                        }
                        EXCEPTION_FLT_OVERFLOW =>
                            eprintln!("[{WAR_COLOR}Warning{RESET_COLOR}] -> A floating point operation resulted in a value too large to represent at address {:#x}", except_addr),

                        EXCEPTION_ILLEGAL_INSTRUCTION => {
                            eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> bad instruction at address {:#x}", except_addr);
                            continue_dbg = false;
                        }
                        EXCEPTION_STACK_OVERFLOW => {
                            eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> stack overflow at address {:#x}", except_addr);
                            continue_dbg = false;
                        }
                        EXCEPTION_ACCESS_VIOLATION => {
                            let access_type = debug_event.u.Exception().ExceptionRecord.ExceptionInformation[0];
                            let drs = debug_event.u.Exception().ExceptionRecord.ExceptionInformation[1];
                            let access_str = match access_type {
                                0 => "read",
                                1 => "write",
                                8 => "execute",
                                _ => "unknown",
                            };
                            let mut mem_info: MEMORY_BASIC_INFORMATION = mem::zeroed();
                            let query_result = VirtualQueryEx(process_handle, drs as LPVOID, &mut mem_info, mem::size_of::<MEMORY_BASIC_INFORMATION>());
                            if query_result == 0 {
                                eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Failed to query memory information : {}", io::Error::last_os_error());
                            } else {
                                eprintln!("[{ERR_COLOR}Critical{RESET_COLOR}] -> memory access violation for '{access_str}' at address {:#x} caused by instruction at address {:#x}", drs, except_addr);
                                memory::mem_info::print_mem_info(mem_info);
                            }
                            continue_dbg = false;
                        }
                        _ => {}
                    }
                }
                CREATE_PROCESS_DEBUG_EVENT => {
                    println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Process created at address: {:x?}", debug_event.u.CreateProcessInfo().lpBaseOfImage);
                    BASE_ADDR = debug_event.u.CreateProcessInfo().lpBaseOfImage as u64;
                    for addr in &OPTION.breakpoint_addr {
                        memory::breakpoint::set_breakpoint(process_handle, *addr);
                    }
                    for addr_over in SKIP_ADDR.clone() {
                        memory::set_addr_over(process_handle, addr_over);
                    }
                    for crt_func in CR_FUNCTION.iter_mut() {
                        memory::set_cr_function(process_handle, crt_func);
                    }
                    for stret in &*stret::BREAK_RET {
                        memory::breakpoint::set_breakpoint_in_ret_func(process_handle, *stret);
                    }
                    memory::watchpoint::set_watchpoint(debug_event, process_handle);
                }
                EXIT_PROCESS_DEBUG_EVENT => {
                    println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Process exited with exit code : {}", debug_event.u.ExitProcess().dwExitCode);
                    continue_dbg = false;
                }
                CREATE_THREAD_DEBUG_EVENT => println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Thread created at address : {:#x}", debug_event.u.CreateThread().lpThreadLocalBase as u64),
                EXIT_THREAD_DEBUG_EVENT => println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Thread exited with exit code : {}", debug_event.u.ExitThread().dwExitCode),
                LOAD_DLL_DEBUG_EVENT => {
                    let dll_base = debug_event.u.LoadDll().lpBaseOfDll;
                    let h_file = debug_event.u.LoadDll().hFile;
                    let mut buffer: [c_char; winapi::shared::minwindef::MAX_PATH] = [0; winapi::shared::minwindef::MAX_PATH];
                    let len = GetFinalPathNameByHandleA(h_file, buffer.as_mut_ptr(), winapi::shared::minwindef::MAX_PATH as u32, 0);
                    if len > 0 {
                        let path = std::slice::from_raw_parts(buffer.as_ptr() as *const u8, len as usize);
                        if let Ok(cstr) = std::str::from_utf8(path) {
                            let display_path = if cstr.starts_with(r"\\?\") {
                                &cstr[4..]
                            } else {
                                cstr
                            };
                            println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Dll at address: {:#x} has been loaded ;{}", dll_base as u64, display_path);
                        } else {
                            println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Dll at address: {:#x} has been loaded", dll_base as u64);
                        }
                    } else {
                        println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Dll at address: {:#x} has been loaded", dll_base as u64);
                    }
                }
                UNLOAD_DLL_DEBUG_EVENT =>  println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Dll at address {:x?} has been unloaded", debug_event.u.UnloadDll().lpBaseOfDll),
                OUTPUT_DEBUG_STRING_EVENT =>  {
                    let dbg_str = debug_event.u.DebugString();
                    let dbg_strd = dbg_str.lpDebugStringData;
                    let c_str = std::ffi::CStr::from_ptr(dbg_strd as *const c_char);
                    let dbg_str = c_str.to_str().unwrap();
                    println!("[{DBG_COLOR}Debug{RESET_COLOR}] -> Debug string output : \"{dbg_str}\"");
                },
                _ => {}
            }
            if continue_dbg {
                if ContinueDebugEvent(debug_event.dwProcessId, debug_event.dwThreadId, DBG_CONTINUE) == 0 {
                    eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> failed to ContinueDebugEvent : {}", io::Error::last_os_error());
                    stop_dbg(process_handle, debug_event);
                }
            }else {
                stop_dbg(process_handle, debug_event);
            }
        }
    }
}




unsafe fn stop_dbg(process_handle: HANDLE, debug_event: DEBUG_EVENT) {
    for crt_func in CR_FUNCTION.iter_mut() {
        let addr_v = crt_func.address as LPVOID;
        if VirtualFreeEx(process_handle, addr_v, 0, MEM_RELEASE) == 0 {
            eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> Error when freeing memory allocated to address {:#x} : {}", crt_func.address, io::Error::last_os_error());
        }
        crt_func.address = 0;
    }
    BASE_ADDR = 0;
    DebugActiveProcessStop(debug_event.dwProcessId);
}





pub fn start_debugging(exe_path: &str) {
    unsafe {
        let mut si = mem::zeroed::<STARTUPINFOA>();
        let mut pi = mem::zeroed::<PROCESS_INFORMATION>();
        si.cb = mem::size_of::<STARTUPINFOA>() as u32;
        let exe_path_cstr = CString::new(exe_path).unwrap();
        if CreateProcessA(ptr::null_mut(), exe_path_cstr.as_ptr() as *mut i8, ptr::null_mut(), ptr::null_mut(), false as i32, DEBUG_PROCESS,
                          ptr::null_mut(), ptr::null_mut(), &mut si, &mut pi) == 0 {
            eprintln!("[{ERR_COLOR}Error{RESET_COLOR}] -> CreateProcess failed : {}", io::Error::last_os_error());
            return;
        }
        debug_loop(pi.hProcess);
        CloseHandle(pi.hProcess);
        CloseHandle(pi.hThread);
    }
}