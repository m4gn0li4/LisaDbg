use winapi::um::winnt::{CONTEXT, HANDLE, WOW64_CONTEXT};
use crate::ALL_ELM;
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
    if unsafe { ALL_ELM.file.is_some() } {
        let arg = unsafe {
            if ALL_ELM.arg.is_some() {
                format!("{} {}", &ALL_ELM.file.clone().unwrap(), &ALL_ELM.arg.clone().unwrap())
            }else {
                ALL_ELM.file.clone().unwrap()
            }
        };
        exec::start_debugging(&arg);
    } else {
        eprintln!("{ERR_COLOR}Please enter a file path{RESET_COLOR}");
    }
}



fn init(h_proc: HANDLE) {
    unsafe {
        for addr in &ALL_ELM.break_rva {
            memory::breakpoint::set_breakpoint(h_proc, *addr + BASE_ADDR);
        }
        for addr in &ALL_ELM.break_ret {
            memory::breakpoint::set_breakpoint(h_proc, *addr +  BASE_ADDR);
        }
        for addr in &ALL_ELM.break_ret_va {
            memory::breakpoint::set_breakpoint(h_proc, *addr + BASE_ADDR);
        }
        for addr_over in &ALL_ELM.skip_addr {
            memory::set_addr_over(h_proc, *addr_over);
        }
        for crt in ALL_ELM.crt_func.iter_mut() {
            memory::func::set_cr_function(h_proc, crt);
        }
    }
}
