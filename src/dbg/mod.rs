use winapi::um::winnt::{CONTEXT, HANDLE, WOW64_CONTEXT};
use crate::command::skip::SKIP_ADDR;
use crate::command::stret;
use crate::OPTION;
use crate::pefile::function::CR_FUNCTION;
use crate::utils::*;

pub mod dbg_cmd;
pub mod memory;
mod handle_point;
mod exec;
pub mod attach;

const STATUS_WX86_BREAKPOINT: u32 = 0x4000001f;
const STATUS_WX86_SINGLE_STEP: u32 = 0x4000001e;


pub static mut SAVEINSN: Vec<SaveInsn> = Vec::new();
pub struct SaveInsn {
    pub addr: u64,
    pub last_oc: u8,
}



pub trait RealAddr {
    fn real_addr64(&self, ctx: CONTEXT) -> u64;
    fn real_addr32(&self, ctx: WOW64_CONTEXT) -> u32;
}



pub static mut BASE_ADDR: u64 = 0;





pub fn run() {
    if unsafe { OPTION.file.is_some() } {
        let arg = unsafe {
            if OPTION.arg.is_some() {
                format!("{} {}", &OPTION.file.clone().unwrap(), &OPTION.arg.clone().unwrap())
            }else {
                OPTION.file.clone().unwrap()
            }
        };
        exec::start_debugging(&arg);
    } else {
        eprintln!("{ERR_COLOR}Please enter a file path{RESET_COLOR}");
    }
}



fn init(p_handle: HANDLE) {
    unsafe {
        for addr in &OPTION.breakpoint_addr {
            memory::breakpoint::set_breakpoint(p_handle, *addr);
        }
        for addr_over in SKIP_ADDR.clone() {
            memory::set_addr_over(p_handle, addr_over);
        }
        for crt_func in CR_FUNCTION.iter_mut() {
            memory::set_cr_function(p_handle, crt_func);
        }
        for stret in &*stret::BREAK_RET {
            memory::breakpoint::set_breakpoint_in_ret_func(p_handle, *stret);
        }
    }
}
