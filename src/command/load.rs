use crate::{OPTION, symbol};

pub fn load(linev: &[&str]) {
    if linev.len() < 3 {
        println!("USAGE: load <element> <arg>");
        return;
    }
    let elm = linev[1];
    match elm {
        "symbol" => load_sym_file(linev[2]),
        _ => {}
    }
}



fn load_sym_file(symbolpath: &str) {
    unsafe {
        let old_path = OPTION.file.clone();
        OPTION.file = Some(symbolpath.to_string());
        symbol::pdb::target_symbol();
        OPTION.file = old_path;
    }
}


