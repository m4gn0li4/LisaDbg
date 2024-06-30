use anyhow::Error;
use crate::dbg::hook::{Hook, ModIntpr};
use crate::{OPTION, symbol, usage};
use crate::dbg::hook;
use crate::log::*;

pub fn handle_hook_func(linev: &[&str]) {
    if linev.len() < 3 {
        eprintln!("{}", usage::USAGE_HOOK);
        return;
    }
    let mut vec_addr_1 = Vec::new();
    let mut mod_1 = ModIntpr::Name;
    for word in linev[1..].iter() {
        if *word == "-a" || *word == "--address" {
            mod_1 = ModIntpr::Address;
            continue
        }
        match get_addr_with_mod(word, mod_1) {
            Ok(addr_tar) => vec_addr_1.push(addr_tar),
            Err(e) => {
                eprintln!("{ERR_COLOR}{e}{RESET_COLOR}");
                return;
            }
        }
        mod_1 = ModIntpr::Name;
    }
    if vec_addr_1.len() == 2 {
        unsafe { OPTION.breakpoint_addr.push(vec_addr_1[0]) }
        unsafe { hook::HOOK_FUNC.push(Hook {
            target: vec_addr_1[0],
            replacen: vec_addr_1[1],
        }) }
        println!("{VALID_COLOR}the execution flow of the function {:#x} will be redirected to the function {:#x}{RESET_COLOR}", vec_addr_1[0], vec_addr_1[1])
    }
    else {
        eprintln!("{ERR_COLOR}you need to put only 2 items {RESET_COLOR}");
    }
}




fn get_addr_with_mod(target: &str, mod_intpr: ModIntpr) -> anyhow::Result<u64, Error> {
    if mod_intpr == ModIntpr::Address{
        Ok(str_to::<u64>(target)?)
    }
    else {
        let addr_func = symbol::target_addr_with_name_sym(target);
        if addr_func == 0 {
            return Err(Error::msg(format!("unknown symbol '{target}'")));
        }
        Ok(addr_func)
    }
}

