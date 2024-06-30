use winapi::um::winnt::{CONTEXT, M128A};
use crate::usage;
use crate::log::*;

pub enum Value {
    U64(u64),
    U128(M128A),
}


pub unsafe fn handle_reg(linev: &[&str], ctx: CONTEXT) {
    let all_registers: Vec<(&str, Value)> = vec![
        ("rax", Value::U64(ctx.Rax)),
        ("rbx", Value::U64(ctx.Rbx)),
        ("rcx", Value::U64(ctx.Rcx)),
        ("rdx", Value::U64(ctx.Rdx)),
        ("rsi", Value::U64(ctx.Rsi)),
        ("rdi", Value::U64(ctx.Rdi)),
        ("rbp", Value::U64(ctx.Rbp)),
        ("rsp", Value::U64(ctx.Rsp)),
        ("rip", Value::U64(ctx.Rip)),
        ("r8", Value::U64(ctx.R8)),
        ("r9", Value::U64(ctx.R9)),
        ("r10", Value::U64(ctx.R10)),
        ("r11", Value::U64(ctx.R11)),
        ("r12", Value::U64(ctx.R12)),
        ("r13", Value::U64(ctx.R13)),
        ("r14", Value::U64(ctx.R14)),
        ("r15", Value::U64(ctx.R15)),
        ("cs", Value::U64(ctx.SegCs as u64)),
        ("ds", Value::U64(ctx.SegDs as u64)),
        ("es", Value::U64(ctx.SegEs as u64)),
        ("fs", Value::U64(ctx.SegFs as u64)),
        ("gs", Value::U64(ctx.SegGs as u64)),
        ("ss", Value::U64(ctx.SegSs as u64)),
        ("lbfrip", Value::U64(ctx.LastBranchFromRip as u64)),
        ("lbtrip", Value::U64(ctx.LastBranchToRip as u64)),
        ("flag", Value::U64(ctx.EFlags as u64)),
        ("xmm0", Value::U128(ctx.u.s().Xmm0)),
        ("xmm1", Value::U128(ctx.u.s().Xmm1)),
        ("xmm2", Value::U128(ctx.u.s().Xmm2)),
        ("xmm3", Value::U128(ctx.u.s().Xmm3)),
        ("xmm4", Value::U128(ctx.u.s().Xmm4)),
        ("xmm5", Value::U128(ctx.u.s().Xmm5)),
        ("xmm6", Value::U128(ctx.u.s().Xmm6)),
        ("xmm7", Value::U128(ctx.u.s().Xmm7)),
        ("xmm8", Value::U128(ctx.u.s().Xmm8)),
        ("xmm9", Value::U128(ctx.u.s().Xmm9)),
        ("xmm10", Value::U128(ctx.u.s().Xmm10)),
        ("xmm11", Value::U128(ctx.u.s().Xmm11)),
        ("xmm12", Value::U128(ctx.u.s().Xmm12)),
        ("xmm13", Value::U128(ctx.u.s().Xmm13)),
        ("xmm14", Value::U128(ctx.u.s().Xmm14)),
        ("xmm15", Value::U128(ctx.u.s().Xmm15)),
        ("mxcsr", Value::U64(ctx.MxCsr as u64))
    ];

    match linev.get(1) {
        Some(&"all_reg") | Some(&"all_register") => {
            for (reg_name, value) in all_registers.iter().filter(|&reg|reg.0.starts_with("r")) {
                match value {
                    Value::U64(v) => println!("{:<4}: {VALUE_COLOR}{:#x}{RESET_COLOR}", reg_name, v),
                    Value::U128(v) => println!("{:<4}: {VALUE_COLOR}{:#x}{:x}{RESET_COLOR}", reg_name, v.Low, v.High),
                }
            }
        }

        Some(&"all_seg") | Some(&"all_segment") => {
            for (reg_name, value) in all_registers.iter().filter(|&reg|reg.0.ends_with("s") && reg.0.len() == 2) {
                match value {
                    Value::U64(v) => println!("{:<3}: {VALUE_COLOR}{:#x}{RESET_COLOR}", reg_name, v),
                    Value::U128(v) => println!("{:<3}: {VALUE_COLOR}{:#x}{:x}{RESET_COLOR}", reg_name, v.Low, v.High),
                }
            }
        }

        Some(&"all_vec") | Some(&"all_vector") => {
            for (reg_name, value) in all_registers.iter().filter(|&reg|reg.0.starts_with("x")) {
                match value {
                    Value::U64(v) => println!("{:<6}: {VALUE_COLOR}{:#x}{RESET_COLOR}", reg_name, v),
                    Value::U128(v) => println!("{:<6}: {VALUE_COLOR}{:#x}{:x}{RESET_COLOR}", reg_name, v.Low, v.High),
                }
            }
        }

        Some(&"all") => {
            for (reg_name, reg_value) in &all_registers {
                match reg_value {
                    Value::U64(v) => println!("{}: {VALUE_COLOR}{:#x}{RESET_COLOR}", reg_name, v),
                    Value::U128(v) => println!("{}: {VALUE_COLOR}{:#x}{:x}{RESET_COLOR}", reg_name, v.Low, v.High),
                }
            }
        }
        Some(register) => {
            let mut found = false;
            for (reg_name, reg_value) in &all_registers {
                if *register == *reg_name {
                    match reg_value {
                        Value::U64(v) => println!("{:<4}: {VALUE_COLOR}{:#x}{RESET_COLOR}", reg_name, v),
                        Value::U128(v) => println!("{:<4}: {VALUE_COLOR}{:#x}{:x}{RESET_COLOR}", reg_name, v.Low, v.High),
                    }
                    found = true;
                    break;
                }
            }
            if !found {
                println!("{ERR_COLOR}Unknown register: {register}{RESET_COLOR}");
            }
        }
        None => {
            println!("{}", usage::USAGE_INFO);
        }
    }
}