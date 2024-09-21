use crate::usage::{help_def, USAGE_B_RET_VA};
use crate::utils::*;
use crate::usage;
use crate::usage::{USAGE_INFO, USAGE_SET_REG, USAGE_MEM_INFO};

pub const USAGE_DEREF: &str = r#"
Usage: deref <type> <address/register>

Description:
    Dereference and read the value at a specific memory address or register in the target process.

Arguments:
    <type>           The data type of the value to dereference. Supported types include:
                     - char = uint8_t      : 8bit unsigned integer
                     - int8_t, uint8_t     : 8bit signed or unsigned integer
                     - int16_t, uint16_t   : 16bit signed or unsigned integer
                     - int32_t, uint32_t   : 32bit signed or unsigned integer
                     - int64_t, uint64_t   : 64bit signed or unsigned integer
                     - char[]              : Null-terminated string (array of characters)

    <address/register>  The memory address or register name to dereference
                        If a register name is provided, its current value will be used as the memory address

Note:
  for arrays, you must specify the number of elements within brackets (e.g., uint64_t[2])
  this will dereference the specified number of values starting from the provided address
  the only exception is char[], which will read a string up to the first null character

- Examples:
    deref uint32_t 0x7ff61a03183a       # Dereference a 32bit unsigned integer at address 0x7ff61a03183a
    deref int64_t rax                   # Dereference a 64bit signed integer using the current value of the rax register
    deref uint64_t[2] rbx               # Dereference two 64bit unsigned integers starting at the address contained in rbx
    deref char[] rsp                    # Dereference and read a string up to the null character from the address contained in rsp (works with all 64bit registers)
"#;






pub const USAGE_DISASM: &str = r#"Usage: disasm <address/register> [count]
Description:
  Disassembles a given number of instructions from a specified address (va) or register
  If the count is not specified, the disassembler will automatically disassemble the function
  in which the address is located

Options:
  <address/register>   The address (virtual address) or register to disassemble
  [count]              The number of instructions to disassemble. If omitted, the entire
                       function containing the address will be disassembled

Examples:
  disasm 0x400000        # Disassemble instructions starting from the address 0x400000, continuing until the end of the function containing this address
  disasm rax 10          # Disassemble 10 instructions starting from the address stored in the rax register
  disasm 0x400000 20     # Disassemble 20 instructions starting from the address 0x400000
"#;

pub const USAGE_SET_MEM: &str = r#"
Usage: set mem <type> <address/register> <new_value>

Description:
    Set the value at a specific memory address in the target process

Arguments:
    <type>            The data type of the value to set. Supported types include:
                        - int8_t,  uint8_t,  char   : 8bit signed or unsigned integer
                        - int16_t, uint16_t, word   : 16bit signed or unsigned integer
                        - int32_t, uint32_t, dword  : 32bit signed or unsigned integer
                        - int64_t, uint64_t, qword  : 64bit signed or unsigned integer


    <address/register>  The memory address or register name whose value will be set.
                        If a register name is provided, its current value will be used as the memory address.

    <new_value>      The new value to write to the specified memory address or register.

Note:
    For arrays, append '[]' to the type (e.g., uint64_t[])
    You can optionally specify the number of elements in parentheses (this is still recommended)
    If the number of provided values is less than the specified number, the script will pad with null values
    If the specified number is less than the number of provided values, the script will only use the number of values specified


- Examples:
    set memory uint32_t 0x7ff61a03183a 0xdeadbeef      # Set a 32bit unsigned integer value at address 0x7ff61a03183a
    set mem int64_t rax 1234567890123456               # Set a 64bit signed integer at the address contained in rax
    set memory uint16_t[4] r14  0x12, 'c', 9, "a"      # Set the values "0x12, 'c', 9, 'a'" at the address contained in r14 (each element is cast to uint16_t here)
    set mem uint64_t[2] rax 0x1400000000               # Set the value 0x1400000000 at the address contained in rax (the script will add 0s to imitate a 2nd value)
    set memory char[] rsp "hello world", 0             # Write a string with a null character to the address contained in rsp
"#;



pub const USAGE_BACKTRACE: &str = r#"Usage: backtrace <count>

The 'backtrace' command prints the call stack frames for debugging purposes.

Parameters:
  <count>  - Specifies the number of frames to display.
             If 'full' is provided, all frames in the call stack will be displayed.

Examples:
  backtrace 5      - Displays the first 5 frames of the call stack.
  backtrace full   - Displays all frames in the call stack.

This command will list each frame in the call stack, helping you understand the sequence of function calls leading to a particular point in the program. This is useful for debugging and tracing the flow of execution.

"#;



pub const USAGE_SA: &str = r#"USAGE: sym-addr <NAME>

Description
  for view the addresse (va) of symbols specified"#;




pub fn help(linev: &[&str]) {
    if linev.len() == 1 {
        println!(r#"{VALID_COLOR}
Available commands:

   c, continue, run            : Continue the execution of the process
   v, value                    : Display the value of a specified register
   s                           : for load the symbol file (if avaible)
   deref                       : Dereference the value at a specific memory address or register in the target process
   q, quit, break              : Terminate the debugging session. Confirmation required
   base-addr, ba               : Display the base address of the target process
   set                         : To set something, it can be a register, a value at an address or a memory protection, to find out more type "help set"
   b, breakpoint               : Set a breakpoint at the specified address (rva) or symbol
   reset                       : Reset the state of the debugging session
   cva                         : Calculates the va of a specified rva
   ret                         : Set the instruction pointer (rip) to the return address of the current function and decrement the stack pointer (rsp) by 8 (only if the function had been specified with stret)
   skip                        : skip calls to the specified function
   break-ret, b-ret            : places a breakpoint at the return address of the specified function (or the function that contains the instruction at the specified address)
   view                        : see certain information like the breakpoints that have been placed etc
   sym-address                 : for view the symbol address with here name (va)
   proc-addr                   : get the address of a function in a dll
   disasm                      : to disassemble opcodes from a specified address (va)
   b-va, break-va              : Sets a breakpoint at the specified address (va)
   backtrace, frame            : for print the call stack frames for debugging purposes
   symbol-local, sym-local     : to display all local symbols relating to the current function (only if the symbol type is pdb)
   def                         : to declare a function or a type or a structure
   address-func, addr-func     : displays current function information
   break-ret-va, b-ret-va      : places a breakpoint at the return address
   mem-info                    : gives all the memory information at this address (base address, state etc.)
   help                        : Display this help message


for more information (if available) just type <command> without its arguments{RESET_COLOR}"#);
    }
    else {
        let arg = linev[1];
        match arg {
            "c" | "continue" | "run" => println!("Continue the execution of the process"),
            "v" | "value" => println!("{}", USAGE_INFO),
            "s" | "symbol" => println!("for load the symbol file (if avaible)"),
            "deref" => println!("{}", USAGE_DEREF),
            "set" => help_set(&linev),
            "b" | "breakpoint" => println!("{}", usage::USAGE_BRPT),
            "reset" => println!("{}", usage::USAGE_RESET),
            "cva" => println!("Calculates the va of a specified rva"),
            "skip" => println!("{}", usage::USAGE_SKIP),
            "break-ret" | "b-ret" => println!("{}", usage::USAGE_B_RET),
            "bva" | "break-va" | "b-va" => println!("USAGE: break-va <Va>"),
            "view" => println!("{}", usage::USAGE_VIEW),
            "proc-addr" => println!("{}", usage::USAGE_PROC_ADDR),
            "sym-addr" | "sym-address" => println!("for view the symbol address with here name (va)"),
            "disasm" => println!("{}", USAGE_DISASM),
            "backtrace" | "frame" => println!("{}", USAGE_BACKTRACE),
            "sym-info" => println!("{}", usage::USAGE_SYM_INFO),
            "def" => help_def(&linev[1..]),
            "break-ret-va" | "b-ret-va" => println!("{USAGE_B_RET_VA}"),
            "mem-info" => println!("{USAGE_MEM_INFO}"),
            "help" => println!("Display this help message"),
            _ => {}
        }
    }
}


fn help_set(linev: &[&str]) {
    if linev.len() > 2 {
        match linev[2] {
            "mem" | "memory" => println!("{}", USAGE_SET_MEM),
            "reg" | "register" => println!("{}", USAGE_SET_REG),
            "mem-protect" | "memory-protection" => println!("{}",usage::USAGE_SET_PROTECT),
            _ => {}
        }
    }else {
        println!("{}", usage::USAGE_SET);
    }
}