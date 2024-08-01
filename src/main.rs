mod command;
mod log;
mod dbg;
mod symbol;
mod ste;
mod pefile;
mod usage;

use std::io;
use std::io::Write;
use structopt::StructOpt;
use once_cell::sync::Lazy;
use crate::command::watchpoint::Watchpts;

#[derive(Debug, StructOpt, Default)]
#[structopt(name = "LisaDbg")]
struct Dbgoption {
    file: Option<String>,
    #[structopt(short = "b", long = "breakpoint", help = "to place a breakpoint at an address (RVA)")]
    breakpoint_addr: Vec<u64>,
    #[structopt(short = "a", long = "arg", help = "set arguments for script to debug")]
    arg: Option<String>,
    #[structopt(long = "exec", help = "for execute a cmd specified before running dbg")]
    exec_cmd: Vec<String>,
    #[structopt(short = "w", long = "watchpoint", help = "Set a watchpoint in the format '[--memory=<zone>] [--access=<rights>] <offset>")]
    watchpts: Vec<Watchpts>
}




impl Dbgoption {
    pub fn exec_cmd(&self) {
        for cmd in &self.exec_cmd {
            let linev: Vec<&str> = cmd.split_whitespace().collect();
            handle_cmd(&linev, &cmd);
        }
    }
}


static mut OPTION: Lazy<Dbgoption> = Lazy::new(||Dbgoption::from_args());


fn main() {
    unsafe { OPTION.exec_cmd() }
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
        Some(&"create-function") | Some(&"create-func") | Some(&"crt-func") => command::create_func::crte_function(&linev),
        Some(&"arg") | Some(&"args") | Some(&"argv") => command::arg::set_argument(&linev, input),
        Some(&"help") | Some(&"h")  => usage::help(&linev),
        Some(&"help-c") => dbg::dbg_cmd::usages::help(&linev),
        Some(&"view") => unsafe {command::viewing::view_brpkt(&linev, std::mem::zeroed())}
        Some(&"w") | Some(&"watch") | Some(&"watchpoint") => command::watchpoint::watchpoint(&linev),
        Some(&"clear") => command::clear_cmd::clear_cmd(),
        Some(&"remove") => command::remover::remove_element(&linev),
        Some(&"sym-info") => unsafe {command::sym::handle_sym_info(&linev, std::mem::zeroed())}
        Some(&"add") => command::little_secret::add_op(&linev),
        Some(&"sub") => command::little_secret::sub_op(&linev),
        None => eprintln!("{}please enter a command{}", log::ERR_COLOR, log::RESET_COLOR),
        _ => eprintln!("{}command '{}' is unknow{}", log::ERR_COLOR, cmd.unwrap(), log::RESET_COLOR)
    }
}


