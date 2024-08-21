use winapi::um::winnt::{CONTEXT, WOW64_CONTEXT};
use crate::OPTION;
use crate::utils::*;

pub mod dbg_cmd;
pub mod memory;
mod handle_point;
mod exec;

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
