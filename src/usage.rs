use crate::log::{ERR_COLOR, RESET_COLOR, VALID_COLOR};

pub const USAGE_SKIP: &str = "\x1b[32mUSAGE: skip [flag] <FUNCTION>\
        \nflag:\
        \n -a   --address   will consider that you had set the rva address of the function to <FUNCTION>\n\x1b[0m";

pub const USAGE_B_RET: &str = "\x1b[32mUSAGE: b-ret [flag] <FUNCTION>

Description:
  to place a breakpoint at all \"ret\" statements of the specified function

flag:
  -a   --address   will consider that you had set the rva address of the function to <FUNCTION>

Example:
  b-ret main          # Places a breakpoint on all ret instructions of the \"main\" function
  b-ret -a 0x1234     # Places a breakpoint on all ret instructions of the function at address 0x1234 (it must be a function at address 0x1234)
\x1b[0m";

pub const USAGE_BRPT: &str = "\x1b[32mUSAGE: breakpoint <RVA-ADDRESS/SYMBOL-NAME>\n
Description:
  To place a breakpoint with the address (rva) or symbol name

Example:
  breakpoint main       # Places a breakpoint at the address of main
  b 0x1234              # Places a breakpoint at the address (base address + 0x1234)

Notes:
   all rva addresses are resolved during the creation of the debug process and are calculated with the base address, if you put the name of a symbol, it will take its rva
\x1b[0m";




pub const USAGE_INFO: &str = "\x1b[32m
Usage: value <option>

Options:
    all-register, all-reg      - Display all general-purpose registers (rax, rbx, rcx, etc..)
    all-segment, all-seg       - Display all segment registers (cs, ds, es, fs, gs, ss)
    all-vector, all-vec        - Display all vector registers (xmm0, xmm1, xmm2, etc..)
    all                        - Display all elements
    <element>                  - Display the specified element

on x64, you can display individual element by specifying their names:
    rax, rbx, rcx, rdx, rsi, rdi, rbp, rsp, rip, r8, r9, r10, r11, r12, r13, r14, r15
    cs, ds, es, fs, gs, ss
    lbfrip, lbtrip, flag
    xmm0, xmm1, xmm2, xmm3, xmm4, xmm5, xmm6, xmm7, xmm8, xmm9, xmm10, xmm11, xmm12, xmm13, xmm14, xmm15
    mxcsr

for x32, it is :
    eax, ebx, ecx, edx, esi, edi, ebp, esp, eip
    cs, segcs, ds, segds, es, seges, fs, segfs, gs, seggs, ss, segss
    flag, eflag
    ctrl-word, control-word, status-word, tag-word, err-offset, error-offset, err-select, error-selector, data-offset, data-selector, data-select

Examples:
    value all-reg        # Display all general-purpose registers
    value all-segment    # Display all segment registers
    value xmm0           # Display the xmm0 register
    value rip            # Display the instruction pointer register
    value rax            # Display the rax register
\x1b[0m";



pub const USAGE_RESET: &str = "\x1b[32m
Usage: reset <option>

Options:
    file                  - Reset the file context
    breakpoint, b         - Clear all breakpoints
    symbol, s             - Clear all symbol loaded
    crt-func              - Clear all function created with cmd 'crt-func'
    hook, ho              - Clear all defined hooks
    b-ret                 - Clear all ret function tracker
    skip                  - Restores the function execution flow defined with \"skip\"
    args                  - Clear the arguments
    watchpoint, watch, w  - Clear all watchpoints
    all                   - Clear all settings and reset to default
\x1b[0m";


pub const USAGE_SYM_INFO: &str = "\x1b[32mUSAGE: sym-info <sym-name>

Description:
  to display all symbol information

Example:
  sym-info main       # Displays information about symbols \"main\"
\x1b[0m";




pub const USAGE_SET_REG: &str = "\x1b[32m
Usage: setr <register> <value>

Description:
    Set the value of a cpu register in the current execution context

Arguments:
    <register>      The name of the register to modify. Supported registers include:
                    - General Purpose Registers: rax, rbx, rcx, rdx, rsi, rdi, rsp, rbp, rip, r8, r9, r10, r11, r12, r13, r14, r15
                    - SIMD Registers: xmm0, xmm1, xmm2, xmm3, xmm4, xmm5, xmm6, xmm7, xmm8, xmm9, xmm10, xmm11, xmm12, xmm13, xmm14, xmm15
                    - Flags Register: flag (Rflags)

    <value>         The new value to set for the specified register. Must be a valid numeric value

Note:
- Examples:
    setr rax 0xDEADBEEF          # Set the value of the rax register to 0xDEADBEEF
    setr xmm0 0xFFFFFFFFFFFFFFF  # Set the value of xmm0 to a floating-point value
    setr flag 0x200              # Set specific bits in the Rflags register
\x1b[0m";




pub const USAGE_HOOK: &str = "\x1b[32m
USAGE: hook [flag] <FUNCTION1> [flag] <FUNCTION2>

Flags:
    -a, --address    Specify that the following argument is an address.

Description:
    This command sets up a hook by redirecting the execution flow from FUNCTION1 to FUNCTION2. Both FUNCTION1 and FUNCTION2 can be specified by name or by address. You can use the -a or --address flag to indicate that the next argument should be interpreted as an address rather than a name.

Examples:
    hook func1 func2
    hook -a 0x12345678 func2
    hook func1 -a 0x12345678
    hook -a 0x12345678 -a 0x87654321

Notes:
    - Exactly two functions must be specified
    - If the address flag is used, ensure that the corresponding argument is a valid address
    - The command will print an error message if the arguments are invalid or insufficient
\x1b[0m";

pub const USAGE_CRT_FUNCTION: &str = "\x1b[32m
create-func <NAME> <RETURN-VALUE>

Description:
    This command schedules a function with a return value that will be initialized when the process is created

Examples:
  create test 1

Notes:
 - you will not know the address of the function before the process starts
\x1b[0m";


pub const USAGE_SET_ARG: &str = "\x1b[32margs <ARGUMENT>
Description:
    This command is for specifying arguments when launching the process to be debugged

Examples:
 args \"--test \"C:\\test\\random_file.bin\"

Notes:
 - if in your cli there must be strings etc and you must put double quotes, then put them
\x1b[0m";




pub const USAGE_VIEW: &str = "\x1b[32mUsage: view <option>
Options:
    breakpoint, brpt, b         Display all breakpoint addresses
    skip                        Display all skip addresses
    b-ret                       Display all b-ret addresses
    symbol, sym, s              Display all symbol
    hook-func, hook, h          Display all hooks that have been defined
    create-func, crt-func       Display all user-created functions
    function, func, f           Display all function entry
    section, sec                Displays section information
    \x1b[0m";






pub const USAGE_WATCHPTS: &str = "\x1b[32m
USAGE: watchpoint '[--memory=<zone>] [--access=<rights>] [--size <size>] <offset>'

Options:
    --memory=<type>    : Specifies the type of memory zone to watch. Available options:
                          - 'stack' : Watches the stack using the offset relative to the RSP (or SP in 32-bit) at the last frame before the current one. The offset is applied to this RSP value and can be negative
                          - (default) 'static', 'static-mem', 'static-memory' : places a watchpoint at the specified offset (the specified offset must be an RVA, it will be calculated subsequently with the base address)
                          - 'virtual', 'virtual-addr', 'virtual-address': virtual address, an exact address in the process's address space

    --access=<rights>   : Specifies the access rights to monitor. Options are:
                          - 'r', 'R' for read (default=RW)
                          - 'w', 'W' for write (defau=RW)
                          - 'x', 'X' for execute


   --register=<register> : the register with which the offset will be calculated (the specified register must be the name of the architecture's extended register)
   --size                : Defines the size of the memory zone to monitor in bytes If not specified can be the size of the type to monitor, (u8=1, u16=2, u32=4, u64=8))
    <offset>             : Offset to apply to the watchpoint address. This can be positive or negative It is a required parameter



Note:
    - if you define the watchpoint on a local variable or from a register, make sure that when you define it you are in the correct scope, if you use a register, make sure that it is initialized at that time

Examples:
   watchpoint --memory=virtual 0x12345678                # Monitor an absolute virtual address with read and write access
   watch --memory=stack --access=r -20                    # Monitor the stack with execute access, where the offset is relative to the RSP value at the last frame
   w --size 2 0x2000                                      # Monitor an RVA address with a specific size
   -w \"register=rbp\"

\x1b[0m";






pub const USAGE_REMOVE: &str = "\x1b[32mUSAGE: remove <element> <rva/symbol-name>

Description :
  to remove an element, specifying the address (rva) or the name of the corresponding symbol


Element:
   breakpoint, b          : To remove a breakpoint
   skip                   : To remove the skip vector function
   b-ret                  : To remove ret monitoring from the function
   hook                   : To remove the hook defined with the specified function (the specified function must be the function that is being replaced)
   watchpoint, watch, w   : To remove a watchpoint defined with the offset (compared to the base address or rsp depending on the selected memory area)
   crt-func               : To remove a function created with its name

Examples:
  remove breakpoint main        # Remove breakpoint set to \"main\"
  remove skip 0xdeadbeef        # Remove address 0xdeadbeef from skip vector
  remove b-ret poop2            # Remove ret monitoring from poop2
  remove hook test              # Remove redirection from this function
  remove crt-func test2         # Remove test2 of crt-func
\x1b[0m";





fn print_help() {
    println!("{VALID_COLOR}LisaDbg Help:");
    println!("Available commands:");
    println!("    {:<38}{}", "breakpoint, b", "Sets a breakpoint at the specified address (rva) or symbol");
    println!("    {:<38}{}", "file", "Change the current file context");
    println!("    {:<38}{}", "run", "Start or resume execution of the debugged program");
    println!("    {:<38}{}", "reset", "Reset the debugger settings or context");
    println!("    {:<38}{}", "remove", "removes a specified element, for more information type \"help remove\"");
    println!("    {:<38}{}", "quit, q, exit", "Exit the debugger");
    println!("    {:<38}{}", "s, sym, symbol", "Load symbols, this will allow commands like \"b-ret\" to be used with the function name directly");
    println!("    {:<38}{}", "b-ret", "places a breakpoint at each ret of the specified function");
    println!("    {:<38}{}", "skip", "skip calls to the specified function");
    println!("    {:<38}{}", "hook, ho", "Setup a function hook to redirect execution flow");
    println!("    {:<38}{}", "create-func, crt-func", "Create a custom function with a return value allocated at execution");
    println!("    {:<38}{}", "view", "see certain information like the symbol that have been placed etc");
    println!("    {:<38}{}", "watchpoint, watch, w", "Set an observation point to a memory location, if the memory location is on the stack, this must be specified");
    println!("    {:<38}{}", "sym-info", "displays all information of the specified symbol");
    println!("    {:<38}{}", "arg, args, argv", "defined the arguments with which the debugger will launch the target program");
    println!("    {:<38}{}", "help-c", "to display the commands available when the program reaches a breakpoint");
    println!("    {:<38}{}", "help, h", "Display this help message");
    println!("\nFor detailed usage, just type help <command> or <command> without its arguments {RESET_COLOR}");
}



fn print_choice(name: &str) {
    match name{
        "breakpoint" | "b" => println!("{}", USAGE_BRPT),
        "file" => println!("{VALID_COLOR}for select a file to debug{RESET_COLOR}"),
        "run" => println!("{VALID_COLOR}Start or resume execution of the debugged program{RESET_COLOR}"),
        "reset" => println!("{}", USAGE_RESET),
        "remove" => println!("{}", USAGE_REMOVE),
        "quit" | "q" | "exit" => println!("{VALID_COLOR}Exit the debugger{RESET_COLOR}"),
        "symbol" | "sym" | "s" => println!("{VALID_COLOR}for load the symbol file (if avaible){RESET_COLOR}"),
        "b-ret" => println!("{}", USAGE_B_RET),
        "skip" => println!("{}", USAGE_SKIP),
        "hook" | "ho" => println!("{}", USAGE_HOOK),
        "create-func" | "crt-func" => println!("{}", USAGE_CRT_FUNCTION),
        "view" => println!("{}", USAGE_VIEW),
        "watchpoint" | "watch" | "w" => println!("{}", USAGE_WATCHPTS),
        "sym-info" => println!("{}", USAGE_SYM_INFO),
        "args" | "argc" | "argv" | "arg" => println!("{}", USAGE_SET_ARG),
        "help-c" => println!("{VALID_COLOR}to display the commands available when the program reaches a breakpoint{RESET_COLOR}"),
        "help" => println!("{VALID_COLOR}to display the commands to do before starting debugging{RESET_COLOR}"),
        _ => eprintln!("{ERR_COLOR}undefined command : {}{RESET_COLOR}", name),
    }
}





pub fn help(linev: &[&str]) {
    if linev.len() < 2 || linev[1] == "all" {
        print_help();
    } else {
        let cmd_name = linev[1];
        print_choice(cmd_name);
    }
}
