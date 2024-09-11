use crate::dbg::RESET_COLOR;
use crate::dbg::ERR_COLOR;
use std::{io, ptr};
use std::ffi::{c_char, CStr};
use winapi::um::debugapi::DebugActiveProcess;
use winapi::um::handleapi::{CloseHandle, INVALID_HANDLE_VALUE};
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winbase::QueryFullProcessImageNameA;
use winapi::um::winnt::{LPSTR, PROCESS_ALL_ACCESS};
use crate::dbg::exec;
use crate::OPTION;



pub unsafe fn attach_dbg(pid: u32) {
    let h_proc = OpenProcess(PROCESS_ALL_ACCESS, 0, pid);
    if h_proc.is_null() || h_proc == INVALID_HANDLE_VALUE {
        eprintln!("{ERR_COLOR}Failed to open pid {pid} : {}", io::Error::last_os_error());
        return;
    }
    let mut path_size = winapi::shared::minwindef::MAX_PATH as u32;
    let mut path_buf = vec![0u8;path_size as usize];
    if QueryFullProcessImageNameA(h_proc, 0, path_buf.as_mut_ptr() as LPSTR, ptr::addr_of_mut!(path_size)) == 0 {
        eprintln!("{ERR_COLOR}Failed to query full process image name : {}{RESET_COLOR}", io::Error::last_os_error());
        return;
    }
    let path_str = unsafe {CStr::from_ptr(path_buf.as_ptr() as *const c_char)}.to_string_lossy();
    println!("{}", path_str);
    OPTION.file = Some(path_str.to_string());
    if let Err(e) = crate::pefile::parse_header() {
        eprintln!("{ERR_COLOR}Error when parsing pe headers: {e}{RESET_COLOR}");
        return;
    }
    if DebugActiveProcess(pid) == 0 {
        eprintln!("{ERR_COLOR}Failed to debug process : {}{RESET_COLOR}", io::Error::last_os_error());
        return;
    }
    exec::debug_loop(h_proc);
    CloseHandle(h_proc);
}