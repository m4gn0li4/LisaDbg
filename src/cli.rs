use once_cell::sync::Lazy;
use structopt::StructOpt;
use crate::command::def;
use crate::command::def::func::CrtFunc;
use crate::command::hook::Hook;
use crate::command::watchpoint::Watchpts;
use crate::{command, handle_cmd};

#[derive(Debug, Default, Copy, Clone)]
pub struct AfterB {
    pub(crate) last_addr_b: u64,
    pub(crate) after_b: u64
}

#[derive(Debug, Default)]
pub struct All {
    pub file: Option<String>,
    pub break_rva: Vec<u64>,
    pub break_va: Vec<u64>,
    pub arg: Option<String>,
    pub watchpts: Vec<Watchpts>,
    pub skip_addr: Vec<u64>,
    pub crt_func: Vec<CrtFunc>,
    pub break_ret: Vec<u64>,
    pub hook: Vec<Hook>,
    pub attach: Option<String>,
    pub struct_def: Vec<def::structs::TypeP>,
    pub after_b: Vec<AfterB>,
    pub break_ret_va: Vec<u64>,
}

impl All {
    pub fn break_contain(&self, addr: u64) -> bool {
        self.break_rva.contains(&addr) || self.break_va.contains(&addr) || self.break_ret.contains(&addr)
    }
}


pub static mut ALL_ELM: Lazy<All> = Lazy::new(|| All::default());


#[derive(Debug, StructOpt, Default)]
#[structopt(name = "LisaDbg", version = "1.7.0")]
pub struct Dbgoption {
    pub(crate) file: Option<String>,
    #[structopt(short = "b", long = "breakpoint", help = "to place a breakpoint at an address (RVA)")]
    breakpoint_addr: Vec<u64>,
    #[structopt(long = "b-ret-va", help = "to place a breakpoint at ret addr of the function which contain the va")]
    b_ret_va: Vec<u64>,
    #[structopt(long = "b-ret", help = "to place a breakpoint at ret addr of the function which contain the rva")]
    b_ret: Vec<u64>,
    #[structopt(long = "b-va", help = "to place a breakpoint at an address (VA) you must know in advance the address going and")]
    b_va: Vec<u64>,
    #[structopt(short = "a", long = "arg", help = "set arguments for script to debug")]
    arg: Option<String>,
    #[structopt(long = "exec", help = "to execute a cmd specified before running dbg")]
    exec_cmd: Vec<String>,
    #[structopt(short = "w", long = "watchpoint", help = "Set a watchpoint in the format '[--memory=<zone>] [--access=<rights>] <offset>")]
    watchpts: Vec<Watchpts>,
    #[structopt(long = "attach", help = "attach the dbg of a existing process with here pid or here name")]
    attach: Option<String>
}





impl Dbgoption {
    pub fn exec_cmd(&self) {
        for cmd in &self.exec_cmd {
            let linev: Vec<&str> = cmd.split_whitespace().collect();
            handle_cmd(&linev, &cmd);
        }
        if let Some(at_str) = &self.attach{
            let line = format!("attach {at_str}");
            command::attach::handle_attach(&line.split_whitespace().collect::<Vec<&str>>())
        }
    }

    pub fn to_all_elm(&self) -> All {
        let mut result = All::default();
        result.file = self.file.clone();
        result.break_rva = self.breakpoint_addr.clone();
        result.arg = self.arg.clone();
        result.watchpts = self.watchpts.clone();
        result.break_va = self.b_va.clone();
        result.break_ret_va = self.b_ret_va.clone();
        result.break_ret = self.b_ret.clone();
        result
    }
}
