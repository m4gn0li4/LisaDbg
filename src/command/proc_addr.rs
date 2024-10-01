use crate::usage::USAGE_PROC_ADDR;
use crate::utils::{ERR_COLOR, RESET_COLOR, VALID_COLOR};
use std::io;
use winapi::um::libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryA};

pub fn handle_get_proc_addr(args: &[&str]) {
    if args.len() < 3 {
        eprintln!("{}", USAGE_PROC_ADDR);
        return;
    }
    unsafe {
        let new_dll = format!("{}\0", args[1].replace("\"", ""));
        let hdll = LoadLibraryA(new_dll.as_ptr() as *const i8);
        if hdll.is_null() {
            eprintln!("{ERR_COLOR}failed to get module handle : {}{RESET_COLOR}", io::Error::last_os_error());
            return;
        }
        let new_func = format!("{}\0", args[2]);
        let addr_func = GetProcAddress(hdll, new_func.as_ptr() as *const i8);
        if addr_func.is_null() {
            eprintln!("{ERR_COLOR}failed to get addr of func: {}{RESET_COLOR}", io::Error::last_os_error());
            return;
        }
        println!("{VALID_COLOR}address of function : {:#x}{RESET_COLOR}", addr_func as u64);
        FreeLibrary(hdll);
    }
}
