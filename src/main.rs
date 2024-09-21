mod command;
mod utils;
mod dbg;
mod symbol;
mod ste;
mod pefile;
mod usage;
mod process;
mod cli;

use std::io;
use std::io::Write;
use structopt::StructOpt;
use winapi::um::winnt::HANDLE;
use crate::cli::ALL_ELM;
use crate::command::def;






fn main() {
    unsafe {
        let option = cli::Dbgoption::from_args();
        *ALL_ELM = option.to_all_elm();
        if let Some(file) = &option.file {
            let intp = format!("file {file}");
            command::file::handle_change_file(&intp.split_whitespace().collect::<Vec<&str>>(), &intp);
        }
        option.exec_cmd();
    }
    ctx_before_run();
}



fn ctx_before_run() {
    loop {
        let mut input = String::new();
        print!("lisa>> ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim_start().trim_end();
        let linev: Vec<&str> = input.split_whitespace().collect();
        handle_cmd(&linev, input);
    }
}




fn handle_cmd(linev: &[&str], input: &str) {
    let cmd = linev.first();
    match cmd {
        Some(&"breakpoint") | Some(&"b")  => command::breakpoint::handle_breakpts(&linev),
        Some(&"file") => command::file::handle_change_file(&linev, input),
        Some(&"run") => dbg::run(),
        Some(&"reset") => command::reset::handle_reset(&linev),
        Some(&"quit") | Some(&"q") | Some(&"exit") => std::process::exit(0),
        Some(&"s") | Some(&"sym") | Some(&"symbol") => symbol::load_symbol(),
        Some(&"break-ret") | Some(&"b-ret") => command::stret::st_return(&linev),
        Some(&"skip") => command::skip::skip(&linev),
        Some(&"hook") | Some(&"ho") => command::hook::handle_hook_func(&linev),
        Some(&"def") => def::handle_def(&linev),
        Some(&"arg") | Some(&"args") | Some(&"argv") => command::arg::set_argument(&linev, input),
        Some(&"help") | Some(&"h")  => usage::help(&linev),
        Some(&"help-c") => dbg::dbg_cmd::usages::help(&linev),
        Some(&"view") => unsafe {command::viewing::view_brpkt(&linev, std::mem::zeroed(), 0 as HANDLE)}
        Some(&"w") | Some(&"watch") | Some(&"watchpoint") => command::watchpoint::watchpoint(&linev),
        Some(&"clear") => command::clear_cmd::clear_cmd(),
        Some(&"remove") => command::remover::remove_element(&linev),
        Some(&"sym-info") => unsafe {command::sym::handle_sym_info(&linev, std::mem::zeroed())}
        Some(&"attach") => command::attach::handle_attach(&linev),
        Some(&"bva") | Some(&"b-va") | Some(&"break-va") => command::breakpoint::handle_break_va(&linev),
        Some(&"proc-addr") => command::proc_addr::handle_get_proc_addr(linev),
        Some(&"b-ret-va") | Some(&"b-retva") => command::stret::handle_b_ret_va(&linev),
        Some(&"add") => command::little_secret::add_op(&linev),
        Some(&"sub") => command::little_secret::sub_op(&linev),
        None => eprintln!("{}please enter a command{}", utils::ERR_COLOR, utils::RESET_COLOR),
        _ => eprintln!("{}command '{}' is unknow{}", utils::ERR_COLOR, cmd.unwrap(), utils::RESET_COLOR)
    }
}


