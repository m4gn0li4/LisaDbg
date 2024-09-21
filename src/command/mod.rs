pub mod breakpoint;
pub mod file;
pub mod reset;
pub mod hook;
pub mod viewing;
pub mod watchpoint;
pub mod sym;
pub mod skip;
pub mod stret;
pub mod remover;
pub mod load;
pub mod attach;
pub mod set;
pub mod def;
pub mod proc_addr;

pub mod arg {
    use crate::{ALL_ELM, usage};
    use crate::utils::*;

    pub fn set_argument(linev: &[&str], line: &str) {
        if linev.len() < 2 {
            println!("{}", usage::USAGE_SET_ARG);
            return;
        }
        unsafe {ALL_ELM.arg = Some(line.replace(linev[0], ""))}
        println!("{VALID_COLOR}the arguments have been recorded\narg expression : {}{RESET_COLOR}", unsafe {ALL_ELM.arg.clone().unwrap()});
    }
}




pub mod with_va {
    use crate::utils::*;
    pub fn handle_calcule_va(linev: &[&str]) {
        if linev.len() != 2 {
            eprintln!("USAGE: cva <rva>");
            return
        }
        match str_to::<u64>(linev[1]) {
            Ok(value) => println!("{VALID_COLOR}VA is : {VALUE_COLOR}{:#x}{RESET_COLOR}", unsafe { crate::dbg::BASE_ADDR } + value),
            Err(e) => eprintln!("{ERR_COLOR}when transforming rva str '{e}' into u64 : {e}{RESET_COLOR}"),
        }
    }


    pub fn handle_calcule_rva(linev: &[&str]) {
        if linev.len() != 2 {
            return;
        }
        match str_to::<u64>(linev[1]) {
            Ok(addr_va) => unsafe{
                if addr_va < crate::dbg::BASE_ADDR {
                    eprintln!("{ERR_COLOR}the specified address cannot be larger than the base address - {:#x} - {:#x}{RESET_COLOR}", addr_va, crate::dbg::BASE_ADDR);
                    return
                }
                println!("RVA is : {VALUE_COLOR}{:#x}{RESET_COLOR}", addr_va - crate::dbg::BASE_ADDR);
            },
            Err(e) => eprintln!("{ERR_COLOR}{e}{RESET_COLOR}"),
        }
    }
}

pub mod clear_cmd {
    use std::process::Command;

    pub fn clear_cmd() {
        Command::new("cmd")
            .args(&["/C", "cls"])
            .status()
            .unwrap();
    }
}



pub mod little_secret {
    use crate::utils::*;

    pub fn sub_op(linev: &[&str]) {
        if linev.len() != 3 {
            println!("{VALUE_COLOR}USAGE: sub <1> <2>      # this is a little secret cmd lol{RESET_COLOR}");
            return;
        }
        match (str_to::<isize>(linev[1]), str_to::<isize>(linev[2])) {
            (Ok(o1), Ok(o2)) => println!("{VALID_COLOR}result: {:#x}", o1 - o2),
            (Err(o1), Ok(_)) => eprintln!("{ERR_COLOR}the first argument is invalid : {o1}"),
            (Ok(_), Err(o2)) => eprintln!("{ERR_COLOR}the 2nd argument is invalid : {o2}"),
            (Err(e), Err(e2)) => eprintln!("{ERR_COLOR}all argument is invalid, 1 : {e} - 2 : {e2}"),
        }
        print!("{RESET_COLOR}");
    }



    pub fn add_op(linev: &[&str]) {
        if linev.len() != 3 {
            println!("{VALID_COLOR}USAGE: add <1> <2>      # this is a little secret cmd lol{RESET_COLOR}");
            return;
        }
        match (str_to::<usize>(linev[1]), str_to::<usize>(linev[2])) {
            (Ok(o1), Ok(o2)) => println!("result: {VALID_COLOR}{:#x}", o1 + o2),
            (Err(o1), Ok(_)) => eprintln!("{ERR_COLOR}the first argument is invalid : {o1}"),
            (Ok(_), Err(o2)) => eprintln!("{ERR_COLOR}the 2nd argument is invalid : {o2}"),
            (Err(e), Err(e2)) => eprintln!("{ERR_COLOR}all argument is invalid, 1 : {e} - 2 : {e2}"),
        }
        print!("{RESET_COLOR}");
    }
}








