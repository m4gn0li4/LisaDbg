use crate::dbg::dbg_cmd::x32::info_reg::ToValue32;
use crate::dbg::dbg_cmd::x64::info_reg::{ToValue, Value};
use crate::dbg::RealAddr;
use crate::symbol::SYMBOLS_V;
use anyhow::Error;
use num::Num;
use std::fmt;
use std::num::ParseIntError;
use std::str::FromStr;
use winapi::um::winnt::{CONTEXT, WOW64_CONTEXT};

pub const ERR_COLOR: &str = "\x1b[31m";
pub const DBG_COLOR: &str = "\x1b[92m";
pub const INSTR_COLOR: &str = "\x1b[92m";

pub const RESET_COLOR: &str = "\x1b[0m";
pub const VALUE_COLOR: &str = "\x1b[96m";
pub const VALID_COLOR: &str = "\x1b[32m";
pub const WAR_COLOR: &str = "\x1B[93m";
pub const BYTES_COLOR: &str = "\x1b[93m";
pub const ADDR_COLOR: &str = "\x1b[95m";
pub const SYM_COLOR: &str = "\x1b[94m";
pub const CYAN_COLOR: &str = "\x1b[36m";
pub const MAGENTA: &str = "\x1b[35m";
pub const BLUE_COLOR: &str = "\x1b[34m";

pub trait NumConvert {
    fn to_u64(&self) -> u64;
    fn from_u64(value: u64) -> Self;
}

impl NumConvert for u32 {
    fn to_u64(&self) -> u64 {
        *self as u64
    }

    fn from_u64(value: u64) -> Self {
        value as u32
    }
}

impl NumConvert for u64 {
    fn to_u64(&self) -> u64 {
        *self
    }

    fn from_u64(value: u64) -> Self {
        value
    }
}

pub trait Print {
    fn print_value(&self) -> String;
}

impl Print for u8 {
    fn print_value(&self) -> String {
        format!("{:#x}", self)
    }
}

impl Print for u16 {
    fn print_value(&self) -> String {
        format!("{:#x}", self)
    }
}

impl Print for u32 {
    fn print_value(&self) -> String {
        format!("{:#x}", self)
    }
}

impl Print for u64 {
    fn print_value(&self) -> String {
        format!("{:#x}", self)
    }
}

impl Print for usize {
    fn print_value(&self) -> String {
        format!("{:#x}", self)
    }
}

impl Print for f32 {
    fn print_value(&self) -> String {
        format!("{}", self)
    }
}

impl Print for f64 {
    fn print_value(&self) -> String {
        format!("{}", self)
    }
}

fn format_value<T: fmt::Debug + Default + Copy + PartialOrd + fmt::Display + fmt::LowerHex, >(value: T) -> String {
    if value < T::default() {
        format!("{}", value)
    } else {
        format!("{:#x}", value)
    }
}

impl Print for i8 {
    fn print_value(&self) -> String {
        format_value(*self)
    }
}

impl Print for i16 {
    fn print_value(&self) -> String {
        format_value(*self)
    }
}

impl Print for i32 {
    fn print_value(&self) -> String {
        format_value(*self)
    }
}

impl Print for i64 {
    fn print_value(&self) -> String {
        format_value(*self)
    }
}

impl Print for bool {
    fn print_value(&self) -> String {
        if *self {
            "true".to_string()
        } else {
            "false".to_string()
        }
    }
}

impl Print for char {
    fn print_value(&self) -> String {
        format!("{}", self)
    }
}

pub trait ToType {
    fn from_str_value(value: &str) -> Result<Self, Error>
    where
        Self: Sized;
    fn from_char(value: char) -> Self
    where
        Self: Sized;
}

impl ToType for u8 {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        str_to(value)
    }
    fn from_char(value: char) -> Self {
        str_to(&(value as u8).to_string()).unwrap()
    }
}

impl ToType for i8 {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        str_to(value)
    }

    fn from_char(value: char) -> Self {
        str_to(&(value as u8).to_string()).unwrap()
    }
}

impl ToType for u16 {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        str_to(value)
    }

    fn from_char(value: char) -> Self {
        str_to(&(value as u8).to_string()).unwrap()
    }
}

impl ToType for i16 {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        str_to(value)
    }

    fn from_char(value: char) -> Self {
        str_to(&(value as u8).to_string()).unwrap()
    }
}

impl ToType for u32 {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        str_to(value)
    }

    fn from_char(value: char) -> Self {
        str_to(&(value as u8).to_string()).unwrap()
    }
}

impl ToType for i32 {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        str_to(value)
    }

    fn from_char(value: char) -> Self {
        str_to(&(value as u8).to_string()).unwrap()
    }
}

impl ToType for u64 {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        str_to(value)
    }

    fn from_char(value: char) -> Self {
        str_to(&(value as u8).to_string()).unwrap()
    }
}

impl ToType for i64 {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        str_to(value)
    }

    fn from_char(value: char) -> Self {
        str_to(&(value as u8).to_string()).unwrap()
    }
}

impl ToType for f32 {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        Ok(f32::from_str(value)?)
    }

    fn from_char(value: char) -> Self {
        f32::from_str(&(value as u8).to_string()).unwrap()
    }
}

impl ToType for f64 {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        Ok(f64::from_str(value)?)
    }

    fn from_char(value: char) -> Self {
        f64::from_str(&(value as u8).to_string()).unwrap()
    }
}

impl ToType for char {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        let c = if value != "" {
            value.as_bytes()[0] as char
        } else {
            '\0'
        };
        Ok(c)
    }

    fn from_char(value: char) -> Self {
        value
    }
}



impl ToType for bool {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        Ok(value == "true")
    }

    fn from_char(value: char) -> Self {
        value == '1'
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct AsChar(u8);

impl fmt::Display for AsChar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt(self, f)
    }
}

fn fmt(selfs: &AsChar, f: &mut fmt::Formatter) -> fmt::Result {
    if selfs.0.is_ascii_graphic() || selfs.0.is_ascii_whitespace() {
        write!(f, "{}", selfs.0 as char)
    } else {
        write!(f, "\\x{:02x}", selfs.0)
    }
}

impl fmt::Debug for AsChar {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt(self, f)
    }
}

impl ToType for AsChar {
    fn from_str_value(value: &str) -> Result<Self, Error> {
        Ok(AsChar(str_to(value)?))
    }

    fn from_char(value: char) -> Self {
        AsChar(value as u8)
    }
}

pub fn str_to<T: Num<FromStrRadixErr = ParseIntError> + FromStr<Err = ParseIntError>>(addr_str: &str) -> Result<T, Error> {
    if addr_str.to_lowercase().starts_with("0x") {
        Ok(T::from_str_radix(&addr_str.replace("0x", ""), 16)?)
    } else if addr_str.to_lowercase().ends_with("h") {
        Ok(T::from_str_radix(&addr_str[..addr_str.len() - 1], 16)?)
    } else {
        Ok(addr_str.parse()?)
    }
}

pub fn get_addr_va(addr_str: &str, ctx: CONTEXT) -> Result<u64, String> {
    match str_to::<u64>(addr_str) {
        Ok(addr) => Ok(addr),
        Err(e) => unsafe {
            if e.to_string().contains("invalid digit") {
                if let Some(sym) = SYMBOLS_V.symbol_file.iter().find(|s| s.name == addr_str) {
                    Ok(sym.real_addr64(ctx))
                } else {
                    match ctx.str_to_value_ctx(addr_str) {
                        Value::U64(addr) => Ok(addr),
                        _ => Err(format!("{}Invalid target: '{addr_str}'{}", ERR_COLOR, RESET_COLOR)),
                    }
                }
            } else {
                Err(format!("{}Invalid target: '{addr_str}'{}", ERR_COLOR, RESET_COLOR))
            }
        },
    }
}



pub fn get_addr_va32(addr_str: &str, ctx: WOW64_CONTEXT) -> Result<u32, String> {
    match str_to::<u32>(addr_str) {
        Ok(addr) => Ok(addr),
        Err(e) => unsafe {
            if e.to_string().contains("invalid digit") {
                if let Some(sym) = SYMBOLS_V.symbol_file.iter().find(|s| s.name == addr_str) {
                    Ok(sym.real_addr32(ctx))
                } else {
                    let ad = ctx.str_to_ctx(addr_str);
                    if ad != 0 {
                        Ok(ad)
                    } else {
                        Err(format!("{}Invalid target: '{addr_str}'{}", ERR_COLOR, RESET_COLOR))
                    }
                }
            } else {
                Err(format!("{}Invalid target: '{addr_str}'{}", ERR_COLOR, RESET_COLOR))
            }
        },
    }
}



pub fn get_addr_br(addr_str: &str) -> Result<u64, String> {
    match str_to::<u64>(addr_str) {
        Ok(value) => Ok(value),
        Err(_) => unsafe {
            if let Some(sym) = SYMBOLS_V.symbol_file.iter().find(|s| s.name == addr_str) {
                if sym.offset > 0 {
                    Ok(sym.offset as u64)
                } else {
                    Err(format!("{ERR_COLOR}the specified symbol cannot have a negative offset{RESET_COLOR}"))
                }
            } else {
                Err(format!("{ERR_COLOR}invalid target : {}{RESET_COLOR}", addr_str))
            }
        },
    }
}
