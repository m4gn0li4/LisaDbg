pub mod breakpoint;
pub mod file;
pub mod reset;
pub mod hook;
pub mod viewing;

pub mod arg {
    use crate::{OPTION, usage};
    use crate::log::*;

    pub fn set_argument(linev: &[&str], line: &str) {
        if linev.len() < 2 {
            println!("{}", usage::USAGE_SET_ARG);
            return;
        }
        unsafe {OPTION.arg = Some(line.replace(linev[0], ""))}
        println!("{VALID_COLOR}the arguments have been recorded\narg expression : {}{RESET_COLOR}", unsafe {OPTION.arg.clone().unwrap()});
    }
}

pub mod create_func {
    use crate::log::*;
    use crate::pefile::function;
    use crate::pefile::function::CrtFunc;
    use crate::usage;

    pub fn crte_function(linev: &[&str]) {
    if linev.len() != 3 {
        eprintln!("{}", usage::USAGE_CRT_FUNCTION);
        return;
    }
    let mut crt_func = CrtFunc::default();
    crt_func.name = linev[1].to_string();
    let ret_value_str = linev[2];
    match str_to::<u64>(ret_value_str) {
        Ok(value) => crt_func.ret_value = value,
        Err(e) => {
            eprintln!("{ERR_COLOR}{e}{RESET_COLOR}");
            return;
        }
    }
    unsafe {function::CR_FUNCTION.push(crt_func.clone())};
    println!("{VALID_COLOR}the function {} will be initialized when the program is launched{RESET_COLOR}", crt_func.name)
    }
}


pub mod with_va {
    use crate::log::*;
    pub fn handle_calcule_va(linev: &[&str]) {
        if linev.len() != 2 {
            eprintln!("USAGE: clva <rva>");
            return
        }
        match str_to::<u64>(linev[1]) {
            Ok(value) => println!("{VALID_COLOR}VA is : {VALUE_COLOR}{:#x}{RESET_COLOR}", unsafe { crate::dbg::BASE_ADDR } + value),
            Err(e) => eprintln!("{ERR_COLOR}when transforming rva str '{e}' into u64 : {e}{RESET_COLOR}"),
        }
    }
}