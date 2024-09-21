use std::{fmt, io};
use std::io::Write;
use std::str::FromStr;
use regex::Regex;
use crate::ALL_ELM;
use crate::pefile::{NtHeaders, NT_HEADER};
use crate::usage::USAGE_DEF_STRUCT;
use crate::utils::{str_to, ERR_COLOR, RESET_COLOR};

#[derive(Debug, Clone)]
pub enum TypeP {
    U8(usize),
    U16(usize),
    U32(usize),
    U64(usize),
    I8(usize),
    I16(usize),
    I32(usize),
    I64(usize),
    F32(usize),
    F64(usize),
    Char(usize),
    Structs(Vec<StructP>, String),
    Bool(usize),
    Ptr(Box<PtrS>, usize),
    Void
}

impl Default for TypeP {
    fn default() -> Self {
        Self::Void
    }
}


#[derive(Debug, Default, Clone)]
pub struct PtrS {
    pub cout_ptr: usize,
    pub type_deref: Box<TypeP>,
}

#[derive(Debug, Default, Clone)]
pub struct StructP {
    pub name_field: String,
    pub type_p: TypeP
}


impl fmt::Display for TypeP {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeP::U8(size) => write_type(f, "uint8_t", *size),
            TypeP::U16(size) => write_type(f, "uint16_t", *size),
            TypeP::U32(size) => write_type(f, "uint32_t", *size),
            TypeP::U64(size) => write_type(f, "uint64_t", *size),
            TypeP::I8(size) => write_type(f, "int8_t", *size),
            TypeP::I16(size) => write_type(f, "int16_t", *size),
            TypeP::I32(size) => write_type(f, "int32_t", *size),
            TypeP::I64(size) => write_type(f, "int64_t", *size),
            TypeP::F32(size) => write_type(f, "float", *size),
            TypeP::F64(size) => write_type(f, "double", *size),
            TypeP::Char(size) => write_type(f, "char", *size),
            TypeP::Bool(size) => write_type(f, "bool", *size),
            TypeP::Ptr(ptr, size) => {
                let mut result = format!("{}", *ptr.type_deref);
                for _ in 0..ptr.cout_ptr {
                    result.push('*');
                }
                if *size != 1 {
                    write!(f, "{}[{}]", result, size)
                } else {
                    write!(f, "{}", result)
                }
            }
            TypeP::Structs(_, name) => write!(f, "struct {}", name),
            TypeP::Void => write!(f, "void"),
        }
    }
}




fn write_type(f: &mut fmt::Formatter<'_>, type_name: &str, size: usize) -> fmt::Result {
    if size != 1 {
        write!(f, "{}[{}]", type_name, size)
    } else {
        write!(f, "{}", type_name)
    }
}





impl TypeP {
    pub fn get_size(&self) -> usize {
        match self {
            TypeP::U8(cout) | TypeP::I8(cout) | TypeP::Bool(cout) | TypeP::Char(cout)  => cout * 1,
            TypeP::U16(cout) | TypeP::I16(cout) => cout * 2,
            TypeP::U32(cout) | TypeP::I32(cout) | TypeP::F32(cout) => cout * 4,
            TypeP::U64(cout) | TypeP::I64(cout) | TypeP::F64(cout) => cout * 8,
            TypeP::Ptr(_, cout) => {
                if let Some(nt) = unsafe {NT_HEADER} {
                    match nt {
                        NtHeaders::Headers32(_) => 4 * cout,
                        NtHeaders::Headers64(_) => 8 * cout,
                    }
                }else {
                    eprintln!("{ERR_COLOR}you have not defined a file, we cannot know if the target architecture is 32b or 64b{RESET_COLOR}");
                    0
                }
            }
            TypeP::Structs(vtypes, _) => {
                let mut result = 0;
                for types in vtypes {
                    result += types.type_p.get_size();
                }
                result
            }
            TypeP::Void => 0
        }
    }

    pub fn get_name_of_struct(&self) -> String {
        match self {
            TypeP::Structs(_, name) => name.clone(),
            _ => "".to_string()
        }
    }

    pub fn get_field_of_struct(&self) -> Vec<StructP> {
        match self {
            TypeP::Structs(field, _) => field.clone(),
            _ => Vec::new(),
        }
    }

    pub fn get_type_with_str(type_str: &str, cout: usize) -> Result<TypeP, String> {
        match type_str {
            "u8" | "uint8_t" | "byte" => Ok(TypeP::U8(cout)),
            "i8" | "int8_t" => Ok(TypeP::I8(cout)),
            "char" => Ok(TypeP::Char(cout)),
            "u16" | "uint16_t" | "word" => Ok(TypeP::U16(cout)),
            "i16" | "int16_t" | "short" => Ok(TypeP::I16(cout)),
            "u32" | "uint32_t" | "dword" => Ok(TypeP::U32(cout)),
            "i32" | "int32_t" | "int" => Ok(TypeP::I32(cout)),
            "u64" | "uint64_t" | "qword" => Ok(TypeP::U64(cout)),
            "i64" | "int64_t" | "long long" => Ok(TypeP::I64(cout)),
            "f32" | "float" => Ok(TypeP::F32(cout)),
            "f64" | "double" => Ok(TypeP::F64(cout)),
            "bool" => Ok(TypeP::Bool(cout)),
            "void" => Ok(TypeP::Void),
            _ => Err(format!("Unknown type p: {}", type_str))
        }
    }
}



impl FromStr for TypeP {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vs: Vec<&str> = s.split_whitespace().collect();
        if vs.len() == 2 {
            let types = vs[0].to_lowercase();
            let mut cout = 1;
            let re = Regex::new(r"\[(.*?)]").unwrap();
            for cap in re.captures_iter(&vs[1]) {
                if let Some(num) = cap.get(1) {
                    match str_to::<usize>(num.as_str()) {
                        Ok(num) => cout = num,
                        Err(e) => return Err(e.to_string())
                    }
                }else {
                    continue
                }
            }
            if types.contains("*") {
                let cout_ptr =  types.matches("*").count();
                let types_d = types.replace("*", "");
                let ptr_type = TypeP::get_type_with_str(&types_d, cout)?;
                return Ok(TypeP::Ptr(Box::new(PtrS {cout_ptr, type_deref: Box::new(ptr_type)}), cout));
            }
            Ok(TypeP::get_type_with_str(&types, cout)?)
        }else {
            Err("valid line is <type> <field name>[cout]".to_string())
        }
    }
}



pub fn def_struct(linev: &[&str]) {
    if linev.len() < 2 {
        println!("{USAGE_DEF_STRUCT}");
        return;
    }
    let mut field_type = Vec::new();
    let mut field_name = Vec::new();
    println!("struct {} {{", linev[1]);
    loop {
        let mut input = String::new();
        print!("    ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input == ":q" {
            print!("\x1B[1A\x1B[2K");
            break;
        }
        match TypeP::from_str(input) {
            Ok(t) => {
                let inp = input.replace(";", "");
                let name = inp.split_whitespace().collect::<Vec<&str>>()[1].split(']').next().unwrap();
                field_name.push(name.to_string());
                field_type.push(t);
                if !input.ends_with(";") {
                    print!("\x1B[1A\x1B[2K");
                    println!("    {input};");
                }
            },
            Err(e) => {
                print!("\x1B[1A\x1B[2K");
                println!("{ERR_COLOR}    {input} # {e}{RESET_COLOR}");
            }
        }
    }
    println!("}}");
    let mut struct_s = Vec::new();
    for i in 0..field_type.len() {
        let field_name = &field_name[i];
        let field_type = &field_type[i];
        struct_s.push(StructP {name_field: field_name.to_string(), type_p: field_type.clone()})
    }
    unsafe {ALL_ELM.struct_def.push(TypeP::Structs(struct_s, linev[1].to_string()));}
}