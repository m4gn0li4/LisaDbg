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





#[derive(Debug, StructOpt, Default)]
#[structopt(name = "LisaDbg")]
struct Dbgoption {
    #[structopt(short = "f", long = "file", help = "to select a file")]
    file: Option<String>,
    #[structopt(short = "b", long = "breakpoint", help = "to place a breakpoint at an address (RVA)")]
    breakpoint_addr: Vec<u64>,
    #[structopt(short = "r", long = "run", help = "to start debugging without using the mini terminal")]
    run: bool,
    #[structopt(short = "a", long = "arg", help = "set arguments for script to debug")]
    arg: Option<String>,
}


static mut OPTION: Lazy<Dbgoption> = Lazy::new(||Dbgoption::from_args());


fn main() {
    if unsafe {OPTION.run} {
        dbg::run();
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
        let cmd = linev.first();
        match cmd {
            Some(&"breakpoint") | Some(&"b")  => command::breakpoint::handle_breakpts(&linev),
            Some(&"retain-breakpoint") | Some(&"rb") => command::breakpoint::handle_retain_breakpoint(&linev),
            Some(&"file") => command::file::handle_change_file(&linev, input),
            Some(&"run") => dbg::run(),
            Some(&"reset") => command::reset::handle_reset(&linev),
            Some(&"quit") | Some(&"q") | Some(&"exit") => std::process::exit(0),
            Some(&"s") | Some(&"sym") | Some(&"symbol") => symbol::load_symbol(),
            Some(&"stret") => ste::st_return(&linev),
            Some(&"skip") => ste::skip(&linev),
            Some(&"dskip") => ste::dskip(&linev),
            Some(&"dret") => ste::dret(&linev),
            Some(&"hook") | Some(&"ho") => command::hook::handle_hook_func(&linev),
            Some(&"create-function") | Some(&"create-func") | Some(&"crt-func") => command::create_func::crte_function(&linev),
            Some(&"arg") | Some(&"args") | Some(&"argv") => command::arg::set_argument(&linev, input),
            Some(&"help") | Some(&"h")  => usage::help(),
            Some(&"view") => command::viewing::view_brpkt(&linev),
            None => eprintln!("{}please enter a command{}", log::ERR_COLOR, log::RESET_COLOR),
            _ => eprintln!("{}command '{}' is unknow{}", log::ERR_COLOR, cmd.unwrap(), log::RESET_COLOR)
        }
    }
}
