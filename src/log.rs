use std::str::FromStr;
use std::num::ParseIntError;
use num::Num;


pub const ERR_COLOR: &str = "\x1b[31m";
pub const DBG_COLOR: &str = "\x1b[92m";
pub const RESET_COLOR: &str = "\x1b[0m";
pub const VALUE_COLOR: &str = "\x1b[96m";
pub const VALID_COLOR: &str = "\x1b[32m";
pub const WAR_COLOR: &str = "\x1B[93m";


pub fn str_to<T: Num<FromStrRadixErr = ParseIntError> + FromStr<Err = ParseIntError>>(addr_str: &str) -> Result<T, ParseIntError> {
    if addr_str.to_lowercase().starts_with("0x") {
        T::from_str_radix(&addr_str[2..], 16)
    }
    else if addr_str.to_lowercase().ends_with("h") {
        T::from_str_radix(&addr_str[..addr_str.len()-1], 16)
    }
    else {
        addr_str.parse()
    }
}

