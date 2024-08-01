use std::str::FromStr;
use std::num::ParseIntError;
use anyhow::Error;
use num::Num;


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




pub fn str_to<T: Num<FromStrRadixErr = ParseIntError> + FromStr<Err = ParseIntError>>(addr_str: &str) -> Result<T, Error> {
    if addr_str.to_lowercase().contains("0x") {
        Ok(T::from_str_radix(&addr_str.replace("0x", ""), 16)?)
    }
    else if addr_str.to_lowercase().ends_with("h") {
        Ok(T::from_str_radix(&addr_str[..addr_str.len()-1], 16)?)
    }
    else {
        Ok(addr_str.parse()?)
    }
}

