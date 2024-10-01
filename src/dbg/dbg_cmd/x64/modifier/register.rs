use crate::dbg::dbg_cmd::x64::info_reg::Value;
use crate::usage;
use crate::utils::str_to;
use crate::utils::*;
use winapi::um::winnt::{CONTEXT, M128A};

pub fn set_register64(linev: &[&str], ctx: &mut CONTEXT) {
    if linev.len() < 2 {
        eprintln!("{}", usage::USAGE_SET_REG);
        return;
    }
    let target = linev[0];
    let value_str = linev[1];
    let value = match str_to::<u64>(value_str) {
        Ok(val) => Value::U64(val),
        Err(_) => match str_to::<u128>(value_str) {
            Ok(val) => {
                let value = M128A {
                    Low: (val & 0xFFFFFFFFFFFFFFFF) as u64,
                    High: (val >> 64) as i64,
                };
                Value::U128(value)
            }
            Err(e) => {
                eprintln!("{ERR_COLOR}error to parse '{value_str}' : {e}{RESET_COLOR}");
                return;
            }
        },
    };

    match target {
        "rax" => set_reg64(&mut ctx.Rax, value),
        "rbx" => set_reg64(&mut ctx.Rbx, value),
        "rcx" => set_reg64(&mut ctx.Rcx, value),
        "rdx" => set_reg64(&mut ctx.Rdx, value),
        "rsi" => set_reg64(&mut ctx.Rsi, value),
        "rdi" => set_reg64(&mut ctx.Rdi, value),
        "rsp" => set_reg64(&mut ctx.Rsp, value),
        "rbp" => set_reg64(&mut ctx.Rbp, value),
        "rip" => set_reg64(&mut ctx.Rip, value),
        "r8" => set_reg64(&mut ctx.R8, value),
        "r9" => set_reg64(&mut ctx.R9, value),
        "r10" => set_reg64(&mut ctx.R10, value),
        "r11" => set_reg64(&mut ctx.R11, value),
        "r12" => set_reg64(&mut ctx.R12, value),
        "r13" => set_reg64(&mut ctx.R13, value),
        "r14" => set_reg64(&mut ctx.R14, value),
        "r15" => set_reg64(&mut ctx.R15, value),
        "xmm0" => set_reg_simd(unsafe { &mut ctx.u.s_mut().Xmm0 }, value),
        "xmm1" => set_reg_simd(unsafe { &mut ctx.u.s_mut().Xmm1 }, value),
        "xmm2" => set_reg_simd(unsafe { &mut ctx.u.s_mut().Xmm2 }, value),
        "xmm3" => set_reg_simd(unsafe { &mut ctx.u.s_mut().Xmm3 }, value),
        "xmm4" => set_reg_simd(unsafe { &mut ctx.u.s_mut().Xmm4 }, value),
        "xmm5" => set_reg_simd(unsafe { &mut ctx.u.s_mut().Xmm5 }, value),
        "xmm6" => set_reg_simd(unsafe { &mut ctx.u.s_mut().Xmm6 }, value),
        "xmm7" => set_reg_simd(unsafe { &mut ctx.u.s_mut().Xmm7 }, value),
        "xmm8" => set_reg_simd(unsafe { &mut ctx.u.s_mut().Xmm8 }, value),
        "xmm9" => set_reg_simd(unsafe { &mut ctx.u.s_mut().Xmm9 }, value),
        "xmm10" => set_reg_simd(unsafe { &mut ctx.u.s_mut().Xmm10 }, value),
        "xmm11" => set_reg_simd(unsafe { &mut ctx.u.s_mut().Xmm11 }, value),
        "xmm12" => set_reg_simd(unsafe { &mut ctx.u.s_mut().Xmm12 }, value),
        "xmm13" => set_reg_simd(unsafe { &mut ctx.u.s_mut().Xmm13 }, value),
        "xmm14" => set_reg_simd(unsafe { &mut ctx.u.s_mut().Xmm14 }, value),
        "xmm15" => set_reg_simd(unsafe { &mut ctx.u.s_mut().Xmm15 }, value),
        "flag" => set_flag(&mut ctx.EFlags, value),
        _ => {
            eprintln!("{ERR_COLOR}Unknown register: {}{RESET_COLOR}", target);
            return;
        }
    }
}

fn set_flag(eflag: &mut u32, value: Value) {
    match value {
        Value::U64(value) => {
            if (u32::MAX as u64) < value {
                eprintln!("{ERR_COLOR}you cannot put a value above 32bits in this destination{RESET_COLOR}")
            }
            *eflag = value as u32
        }
        Value::U128(_) => eprintln!("{ERR_COLOR}you cannot put a value above 32bits in this destination{RESET_COLOR}"),
        Value::Un => eprintln!("{ERR_COLOR}unknow register{RESET_COLOR}"),
    }
}

fn set_reg_simd(reg: &mut M128A, value: Value) {
    match value {
        Value::U64(value) => {
            reg.Low = value;
            reg.High = 0;
        }
        Value::U128(value) => *reg = value,
        _ => {}
    }
}

fn set_reg64(reg: &mut u64, value: Value) {
    match value {
        Value::U64(val) => *reg = val,
        Value::U128(_) => eprintln!("{ERR_COLOR}you can't put a 128bit value into a 64bit register{RESET_COLOR}"),
        _ => {}
    }
}
