use crate::usage::USAGE_DEF;
use crate::utils::*;

pub mod func;
pub mod structs;

pub fn handle_def(linev: &[&str]) {
    if linev.len() < 2 {
        println!("{USAGE_DEF}");
        return;
    }
    let type_elm = linev[1];
    match type_elm {
        "func" | "function" => func::crt_func(&linev[1..]),
        "struct" => structs::def_struct(&linev[1..]),
        _ => eprintln!("{ERR_COLOR}unknow element '{type_elm}'{RESET_COLOR}"),
    }
}
