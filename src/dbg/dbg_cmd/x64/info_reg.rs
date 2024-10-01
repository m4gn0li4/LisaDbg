use crate::usage;
use crate::utils::*;
use winapi::um::winnt::{CONTEXT, M128A};

pub enum Value {
    U64(u64),
    U128(M128A),
    Un,
}

fn unsigned_to_signed(value: u64) -> i64 {
    if value << 63 != 0 {
        value as i64
    } else if value << 31 != 0 {
        value as i32 as i64
    } else if value << 15 != 0 {
        value as i16 as i64
    } else if value << 7 != 0 {
        value as i8 as i64
    } else {
        value as i64
    }
}

pub trait ToValue {
    fn str_to_value_ctx(self, target: &str) -> Value;
}

impl ToValue for CONTEXT {
    fn str_to_value_ctx(self, target: &str) -> Value {
        let target = target.to_lowercase();
        unsafe {
            match target.as_str() {
                "rax" => Value::U64(self.Rax),
                "rbx" => Value::U64(self.Rbx),
                "rcx" => Value::U64(self.Rcx),
                "rdx" => Value::U64(self.Rdx),
                "rsi" => Value::U64(self.Rsi),
                "rdi" => Value::U64(self.Rdi),
                "rbp" => Value::U64(self.Rbp),
                "rsp" => Value::U64(self.Rsp),
                "rip" => Value::U64(self.Rip),
                "r8" => Value::U64(self.R8),
                "r9" => Value::U64(self.R9),
                "r10" => Value::U64(self.R10),
                "r11" => Value::U64(self.R11),
                "r12" => Value::U64(self.R12),
                "r13" => Value::U64(self.R13),
                "r14" => Value::U64(self.R14),
                "r15" => Value::U64(self.R15),
                "segcs" | "cs" => Value::U64(self.SegCs as u64),
                "segds" | "ds" => Value::U64(self.SegDs as u64),
                "seges" | "es" => Value::U64(self.SegEs as u64),
                "segfs" | "fs" => Value::U64(self.SegFs as u64),
                "seggs" | "gs" => Value::U64(self.SegGs as u64),
                "segss" | "ss" => Value::U64(self.SegSs as u64),
                "eflags" | "flags" => Value::U64(self.EFlags as u64),
                "xmm0" => Value::U128(self.u.s().Xmm0),
                "xmm1" => Value::U128(self.u.s().Xmm1),
                "xmm2" => Value::U128(self.u.s().Xmm2),
                "xmm3" => Value::U128(self.u.s().Xmm3),
                "xmm4" => Value::U128(self.u.s().Xmm4),
                "xmm5" => Value::U128(self.u.s().Xmm5),
                "xmm6" => Value::U128(self.u.s().Xmm6),
                "xmm7" => Value::U128(self.u.s().Xmm7),
                "xmm8" => Value::U128(self.u.s().Xmm8),
                "xmm9" => Value::U128(self.u.s().Xmm9),
                "xmm10" => Value::U128(self.u.s().Xmm10),
                "xmm11" => Value::U128(self.u.s().Xmm11),
                "xmm12" => Value::U128(self.u.s().Xmm12),
                "xmm13" => Value::U128(self.u.s().Xmm13),
                "xmm14" => Value::U128(self.u.s().Xmm14),
                "xmm15" => Value::U128(self.u.s().Xmm15),
                "mxcsr" => Value::U64(self.MxCsr as u64),
                _ => Value::Un,
            }
        }
    }
}

pub const ALL_REG64: [&str; 48] = [
    "rax", "rbx", "rcx", "rdx", "rsi", "rdi", "rbp", "rsp", "rip", "r8", "r9", "r10", "r11", "r12",
    "r13", "r14", "r15", "segcs", "segds", "seges", "segfs", "seggs", "segss", "eflags", "cs",
    "ds", "es", "fs", "gs", "ss", "flags", "xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6",
    "xmm7", "xmm8", "xmm9", "xmm10", "xmm11", "xmm12", "xmm13", "xmm14", "xmm15", "mxcsr",
];

pub unsafe fn handle_reg(linev: &[&str], ctx: CONTEXT) {
    match linev.get(1) {
        Some(&"all-reg") | Some(&"all-register") => {
            for reg_name in ALL_REG64.iter().filter(|&reg| reg.starts_with("r")) {
                let value = ctx.str_to_value_ctx(reg_name);
                match value {
                    Value::U64(v) => {
                        println!("{:<4} = {VALUE_COLOR}{:>#18x}{RESET_COLOR} | {VALUE_COLOR}{:>20}{RESET_COLOR} | {VALUE_COLOR}{:>20}{RESET_COLOR}", reg_name, v, unsigned_to_signed(v), v);
                    }
                    Value::U128(v) => println!("{:<4} = {VALUE_COLOR}{:>#18x}{:x}{RESET_COLOR}", reg_name, v.Low, v.High),
                    _ => {}
                }
            }
        }

        Some(&"all-seg") | Some(&"all-segment") => {
            for reg_name in ALL_REG64.iter().filter(|&reg| reg.ends_with("s") && reg.len() == 2) {
                let value = ctx.str_to_value_ctx(reg_name);
                match value {
                    Value::U64(v) => {
                        println!("{:<3} = {VALUE_COLOR}{:>#18x}{RESET_COLOR}", reg_name, v)
                    }
                    _ => {}
                }
            }
        }

        Some(&"all-vec") | Some(&"all-vector") => {
            for reg_name in ALL_REG64.iter().filter(|r| !r.starts_with("xmm")) {
                let value = ctx.str_to_value_ctx(reg_name);
                match value {
                    Value::U128(v) => println!("{:<6} = {VALUE_COLOR}{:>#x}{:x}{RESET_COLOR} | {VALUE_COLOR}{}{RESET_COLOR}", reg_name, v.Low, v.High, v.Low as f64),
                    _ => {}
                }
            }
        }

        Some(&"all") => {
            for reg_name in ALL_REG64 {
                let reg_value = ctx.str_to_value_ctx(reg_name);
                match reg_value {
                    Value::U64(v) => {
                        println!("{:<6} = {VALUE_COLOR}{:>#18x}{RESET_COLOR} | {VALUE_COLOR}{:>20}{RESET_COLOR} | {VALUE_COLOR}{:>20}{RESET_COLOR}", reg_name, v, unsigned_to_signed(v), v);
                    }
                    Value::U128(v) => println!("{:<6} = {VALUE_COLOR}{:#x}{:x}{RESET_COLOR} | {VALUE_COLOR}{:>20}{RESET_COLOR} | {VALUE_COLOR}{:>20}{RESET_COLOR}", reg_name, v.Low, v.High, v.Low as f32, v.Low as f64),
                    Value::Un => eprintln!("{ERR_COLOR}unknow register : '{reg_name}'{RESET_COLOR}"),
                }
            }
        }

        Some(register) => {
            let reg_value = ctx.str_to_value_ctx(register);
            match reg_value {
                Value::U64(v) => {
                    let signed_v = unsigned_to_signed(v);
                    println!("{:<5} = {VALUE_COLOR}{:#x}{RESET_COLOR} | {VALUE_COLOR}{}{RESET_COLOR} | {VALUE_COLOR}{}{RESET_COLOR}", register, v, signed_v, v);
                }
                Value::U128(v) => println!("{:<5} = {VALUE_COLOR}{:#x}{:x}{RESET_COLOR} | {VALUE_COLOR}{}{RESET_COLOR} | {VALUE_COLOR}{}{RESET_COLOR}", register, v.Low, v.High, v.Low as f32, v.Low as f64),
                Value::Un => eprintln!("{ERR_COLOR}unknow register : '{register}'{RESET_COLOR}")
            }
        }
        None => println!("{}", usage::USAGE_INFO),
    }
}
