use std::ffi::{c_char, CStr};
use std::{io, mem};
use winapi::um::handleapi::CloseHandle;
use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot, Process32First, Process32Next, PROCESSENTRY32, TH32CS_SNAPPROCESS,
};

pub fn get_pid_with_name(name: &str) -> Result<u32, String> {
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    if snapshot.is_null() {
        return Err(format!("failed to create tool snashot : {}", io::Error::last_os_error()));
    }
    let mut entry: PROCESSENTRY32 = unsafe { mem::zeroed() };
    entry.dwSize = size_of::<PROCESSENTRY32>() as u32;
    if unsafe { Process32First(snapshot, &mut entry) } == 0 {
        return Err(format!("failed to get first process entry : {}", io::Error::last_os_error()));
    }
    loop {
        let name_proc = unsafe { CStr::from_ptr(entry.szExeFile.as_ptr() as *const c_char) }.to_string_lossy();
        if name_proc == name {
            unsafe {
                CloseHandle(snapshot);
            }
            return Ok(entry.th32ProcessID);
        }
        if unsafe { Process32Next(snapshot, &mut entry) } == 0 {
            break;
        }
    }
    unsafe {
        CloseHandle(snapshot);
    }
    Err("Process not found".to_string())
}
