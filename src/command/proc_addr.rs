use std::io;
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};
use crate::usage::USAGE_PROC_ADDR;
use crate::utils::{ERR_COLOR, RESET_COLOR, VALID_COLOR};




pub fn handle_get_proc_addr(args: &[&str]) {
    if args.len() < 3 {
        eprintln!("{}", USAGE_PROC_ADDR);
        return;
    }
    unsafe {
        let new_dll = format!("{}\0", args[1]);
        let hdll = GetModuleHandleA(new_dll.as_ptr() as *const i8);
        if hdll.is_null() {
            eprintln!("{ERR_COLOR}failed to get module handle : {}{RESET_COLOR}", io::Error::last_os_error());
            return;
        }
        let new_dll = format!("{}\0", args[2]);
        let addr_func = GetProcAddress(hdll, new_dll.as_ptr() as *const i8);
        if addr_func.is_null() {
            eprintln!("{ERR_COLOR}failed to get addr of func: {}{RESET_COLOR}", io::Error::last_os_error());
            return;
        }
        println!("{VALID_COLOR}address of function : {:#x}{RESET_COLOR}", addr_func as u64);
    }
}
