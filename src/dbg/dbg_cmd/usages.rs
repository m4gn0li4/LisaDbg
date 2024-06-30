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


pub const USAGE_DISASM: &str = r#"Usage: disasm <address> <count>
Description:
  disassembles a given number of instructions from a specified address (va)
"#;

pub const USAGE_SET_MEM: &str = r#"
Usage: setm <type> <address/register> <new_value>

Description:
    Set the value at a specific memory address or register in the target process.

Arguments:
    <type>            The data type of the value to set. Supported types include:
                        - int8_t, uint8_t     : 8bit signed or unsigned integer
                        - int16_t, uint16_t   : 16bit signed or unsigned integer
                        - int32_t, uint32_t   : 32bit signed or unsigned integer
                        - int64_t, uint64_t   : 64bit signed or unsigned integer
                        - char = uint8_t      : 8bit unsigned integer

    <address/register>  The memory address or register name whose value will be set.
                        If a register name is provided, its current value will be used as the memory address.

    <new_value>      The new value to write to the specified memory address or register.

Note:
    For arrays, append '[]' to the type (e.g., uint64_t[])
    You can optionally specify the number of elements in parentheses (this is still recommended)
    If the number of provided values is less than the specified number, the script will pad with null values
    If the specified number is less than the number of provided values, the script will only use the number of values specified


- Examples:
    setm uint32_t 0x7ff61a03183a 0xdeadbeef      # Set a 32bit unsigned integer value at address 0x7ff61a03183a
    setm int64_t rax 1234567890123456            # Set a 64bit signed integer at the address contained in rax
    setm uint16_t[4] r14  0x12, 'c', 9, "a"      # Set the values "0x12, 'c', 9, 'a'" at the address contained in r14 (each element is cast to uint16_t here)
    setm uint64_t[2] rax 0x1400000000            # Set the value 0x1400000000 at the address contained in rax (the script will add 0s to imitate a 2nd value)
    setm char[] rsp "hello world", 0             # Write a string with a null character to the address contained in rsp
"#;


pub fn help() {
    println!(r#"
Available commands:

   c, continue, run            : Continue the execution of the process
   v, value                    : Display the value of a specified register
   deref                       : Dereference the value at a specific memory address or register in the target process
   setr, setreg                : Set a new value to a specified register
   q, quit, break              : Terminate the debugging session. Confirmation required
   base-addr, ba               : Display the base address of the target process
   setm, setmemory             : Defined a new value at the specified memory address (va) or at the specified register (the register value will be considered as address)
   b, breakpoint               : Set a breakpoint at the specified address (rva) or symbol
   rb                          : Restore the byte at the specified address (rva) that was overwritten by a breakpoint
   reset                       : Reset the state of the debugging session
   cva                         : Calculates the va of a specified rva
   b-sp                        : Display the stack pointer (rsp) value at the beginning of the function (only if the function had been specified with stret)
   ret-addr, raddr             : Display the return address of the current function (only if the function had been specified with stret)
   ret                         : Set the instruction pointer (rip) to the return address of the current function and decrement the stack pointer (rsp) by 8 (only if the function had been specified with stret)
   skip                        : skip calls to the specified function
   stret                       : places a breakpoint at each ret of the specified function
   dret                        : removes the specified function from the "stret" field
   dskip                       : removes the specified function from the "skip" field
   view                        : see certain information like the breakpoints that have been placed etc
   disasm                      : to disassemble opcodes from a specified address (va)
   help                        : Display this help message


for more information (if available) just type <command> without its arguments"#);
}