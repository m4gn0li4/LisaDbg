use crate::usage;
use crate::usage::{ca_help, help_def, USAGE_B_RET_VA};
use crate::usage::{USAGE_INFO, USAGE_MEM_INFO, USAGE_SET_REG};
use crate::utils::*;

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

pub const USAGE_DBG_T: &str = "Usage: debug-thread <thread_id>

Description:
    This command changes the current debug thread context. It requires the thread ID (numeric value) of the target thread.

Arguments:
    <thread_id> : The ID of the target thread.

Example:
    debug-thread 1234
    dbg-thread 0x4D2
    dbg-th 5678

Notes:
    - Make sure to provide a valid thread ID.
    - If the thread ID is invalid or the thread cannot be opened, an error message will be displayed.
    - Once the thread is successfully changed, the new thread context will be used for subsequent debugging commands.";

pub const USAGE_SA: &str = r#"USAGE: sym-addr <NAME>

Description
  for view the addresse (va) of symbols specified"#;

pub const USAGE_FIND: &str = r#"find <types> <begin-addr> <end-addr> <values>

Description:
The command allows you to search for a sequence of bytes or values in a specific process memory range by specifying the start and end addresses. You can define the data type to search for, such as unsigned integers, characters, or specific types like `uint32_t`, etc.

Arguments:
  <types>           The data type to search for. You can specify standard types like 'uint8_t', 'int32_t', 'char', etc
  <start-addr>      The memory address where the search begins (in hexadecimal or decimal format).
  <end-addr>        The memory address where the search ends (in hexadecimal or decimal format).
  <values>          Sequence of values to search for, separated by commas. You can search for strings or numeric values.

Supported types:
  - uint8_t, u8, byte
  - int8_t, i8
  - char
  - uint16_t, u16, word
  - int16_t, i16, short
  - uint32_t, u32, dword
  - int32_t, i32, int
  - uint64_t, u64, qword
  - int64_t, i64, long

Examples:
     find uint8_t 0x1000 0x2000 0x90                           # Search for the value 0x90 in a memory range
     find char 0x1000 0x2000 "test"                            # Search for the string "test" in a memory range
     find uint32_t 0x3000 0x4000 0xdeadbeef, 0xcafebabe        # Search for multiple numeric values

Notes:
- If the end address is specified as `0`, the command will use the size of the memory region where the start address is located to determine the search size.
- The sequence to search can consist of numeric values or strings. Strings should be enclosed in either double quotes `" "` or single quotes `' '`.
"#;



pub fn help(linev: &[&str]) {
    if linev.len() == 1 {
        println!(
            r#"{VALID_COLOR}
Available commands:

   address-func, addr-func     : displays current function information
   backtrace, frame            : for print the call stack frames for debugging purposes
   base-addr, ba               : Display the base address of the target process
   b, breakpoint               : Set a breakpoint at the specified address (rva) or symbol
   b-ret                       : places a breakpoint at the return address of the specified function (or the function that contains the instruction at the specified address)
   b-va, break-va              : Sets a breakpoint at the specified address (va)
   break-ret, b-ret            : places a breakpoint at the return address of the specified function (or the function that contains the instruction at the specified address)
   break-ret-va, b-ret-va      : places a breakpoint at the return address
   continue, c, run            : Continue the execution of the process
   cva                         : Calculates the va of a specified rva
   deref                       : Dereference the value at a specific memory address or register in the target process
   dbg-thread, dbg-th          : to debug a thread specified with its id
   disasm                      : to disassemble opcodes from a specified address (va)
   find                        : looking for a value that could be stored between 2 addresses
   help                        : Display this help message
   mem-info                    : gives all the memory information at this address (base address, state etc.)
   proc-addr                   : get the address of a function in a dll
   quit, q, break              : Terminate the debugging session. Confirmation required
   reset                       : Reset the state of the debugging session
   ret                         : Set the instruction pointer (rip) to the return address of the current function and decrement the stack pointer (rsp) by 8 (only if the function had been specified with stret)
   set                         : To set something, it can be a register, a value at an address or a memory protection, to find out more type "help set"
   skip                        : skip calls to the specified function
   s                           : for load the symbol file (if available)
   sym-address                 : for view the symbol address with here name (va)
   symbol-local, sym-local     : to display all local symbols relating to the current function (only if the symbol type is pdb)
   thread-info, th-info        : get information about the current thread being debugged
   value, v, register, reg, r  : Display the value of a specified register
   view                        : see certain information like the breakpoints that have been placed etc


 you can type "help all <element-name>" to know all the commands associated with the element
  <element>:
     b, break, breakpoint
     run
     reset, remove
     ret
     thread, th
     register, reg


for more information (if available) just type <command> without its arguments{RESET_COLOR}"#
        );
    }else if linev[1] == "all" {
        ca_help(linev[2])
    } else {
        let arg = linev[1];
        match arg {
            "backtrace" | "frame" => println!("{}", USAGE_BACKTRACE),
            "b" | "breakpoint" => println!("{}", usage::USAGE_BRPT),
            "break-ret" | "b-ret" => println!("{}", usage::USAGE_B_RET),
            "break-ret-va" | "b-ret-va" => println!("{USAGE_B_RET_VA}"),
            "bva" | "break-va" | "b-va" => println!("USAGE: break-va <Va>"),
            "c" | "continue" | "run" => println!("Continue the execution of the process"),
            "cva" => println!("Calculates the va of a specified rva"),
            "dbg-thread" | "dbg-th" => println!("{USAGE_DBG_T}"),
            "def" => help_def(&linev[1..]),
            "disasm" => println!("{}", USAGE_DISASM),
            "deref" => println!("{}", USAGE_DEREF),
            "find" => println!("{}", USAGE_FIND),
            "help" | "help-c" => println!("Display the help message"),
            "mem-info" => println!("{USAGE_MEM_INFO}"),
            "proc-addr" => println!("{}", usage::USAGE_PROC_ADDR),
            "reset" => println!("{}", usage::USAGE_RESET),
            "s" | "symbol" => println!("for load the symbol file (if available)"),
            "set" => help_set(&linev),
            "skip" => println!("{}", usage::USAGE_SKIP),
            "sym-addr" | "sym-address" => println!("for view the symbol address with here name (va)"),
            "sym-info" => println!("{}", usage::USAGE_SYM_INFO),
            "thread-info" | "th-info" => println!("get information about the current thread being debugged"),
            "value" | "v" | "register" | "registers" | "r" => println!("{}", USAGE_INFO),
            "view" => println!("{}", usage::USAGE_VIEW),
            _ => {}
        }
    }
}




fn help_set(linev: &[&str]) {
    if linev.len() > 2 {
        match linev[2] {
            "mem" | "memory" => println!("{}", USAGE_SET_MEM),
            "reg" | "register" => println!("{}", USAGE_SET_REG),
            "mem-protect" | "memory-protection" => println!("{}", usage::USAGE_SET_PROTECT),
            _ => {}
        }
    } else {
        println!("{}", usage::USAGE_SET);
    }
}
