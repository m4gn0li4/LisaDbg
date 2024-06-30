pub const USAGE_SKIP: &str = "USAGE: skip [flag] <FUNCTION>\
        \nflag:\
        \n -a   --address   will consider that you had set the rva address of the function to <FUNCTION>\n";



pub const USAGE_STRET: &str = "USAGE: stret [flag] <FUNCTION>\
        \nflag:\
        \n -a   --address   will consider that you had set the rva address of the function to <FUNCTION>\n";


pub const USAGE_BRPT: &str = "USAGE: breakpoint <RVA-ADDRESS/SYMBOL-NAME>";



pub const USAGE_INFO: &str = r#"
Usage: value <option>

Options:
    all_reg        - Display all general-purpose registers (rax, rbx, rcx, etc..)
    all_register   - Alias for 'all_reg'
    all_seg        - Display all segment registers (cs, ds, es, fs, gs, ss)
    all_segment    - Alias for 'all_seg'
    all_vec        - Display all vector registers (xmm0, xmm1, xmm2, etc..)
    all_vector     - Alias for 'all_vec'
    all            - Display all elements
    <element>     - Display the specified element

You can display individual element by specifying their names:
    rax, rbx, rcx, rdx, rsi, rdi, rbp, rsp, rip, r8, r9, r10, r11, r12, r13, r14, r15
    cs, ds, es, fs, gs, ss
    lbfrip, lbtrip, flag
    xmm0, xmm1, xmm2, xmm3, xmm4, xmm5, xmm6, xmm7, xmm8, xmm9, xmm10, xmm11, xmm12, xmm13, xmm14, xmm15
    mxcsr

Examples:
    value all_reg        # Display all general-purpose registers
    value all_segment    # Display all segment registers
    value xmm0           # Display the xmm0 register
    value rip            # Display the instruction pointer register
    value rax            # Display the rax register
    "#;




pub const USAGE_RESET: &str = r#"
Usage: reset <option>

Options:
    file            - Reset the file context
    breakpoint, b   - Clear all breakpoints
    symbol, s       - Clear all symbol loaded
    crt-func        - Clear all function created with cmd 'crt-func'
    hook, ho        - Clear all defined hooks
    stret           - Clear all ret function tracker
    stover          - Restores the function execution flow defined with "stover"
    args            - Clear the arguments
    all             - Clear all settings and reset to default
"#;




pub const USAGE_SET_REG: &str = r#"
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
"#;



pub const USAGE_HOOK: &str = "
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
";


pub const USAGE_CRT_FUNCTION: &str = r#"
create-func <NAME> <RETURN-VALUE>

Description:
    This command schedules a function with a return value that will be initialized when the process is created

Examples:
  create test 1

Notes:
 - you will not know the address of the function before the process starts"#;


pub const USAGE_SET_ARG: &str = r#"args <ARGUMENT>
Description:
    This command is for specifying arguments when launching the process to be debugged

Examples:
 args -f "C:\code\rust\LisaDbg\target\debug\LisaDbg.exe"

Notes:
 - if in your cli there must be strings etc and you must put double quotes, then put them
"#;



pub const USAGE_VIEW: &str = "Usage: view_brpkt <option>
Options:
    breakpoint, brpt, b         Display all breakpoint addresses
    skip                        Display all skip addresses
    stret                       Display all stret addresses";


pub fn help() {
    println!("LisaDbg Help:");
    println!("Available commands:");
    println!("    {:<38}{}", "breakpoint, b", "Sets a breakpoint at the specified address (rva) or symbol");
    println!("    {:<38}{}", "retain-breakpoint, rb", "Remove a breakpoint with its RVA address");
    println!("    {:<38}{}", "file", "Change the current file context");
    println!("    {:<38}{}", "run", "Start or resume execution of the debugged program");
    println!("    {:<38}{}", "reset", "Reset the debugger settings or context");
    println!("    {:<38}{}", "quit, q, exit", "Exit the debugger");
    println!("    {:<38}{}", "s, sym, symbol", "Load symbols, this will allow commands like \"stret\" to be used with the function name directly");
    println!("    {:<38}{}", "stret", "places a breakpoint at each ret of the specified function");
    println!("    {:<38}{}", "skip", "skip calls to the specified function");
    println!("    {:<38}{}", "hook, ho", "Setup a function hook to redirect execution flow");
    println!("    {:<38}{}", "dret", "removes the specified function from the \"stret\" field");
    println!("    {:<38}{}", "dskip", "removes the specified function from the \"skip\" field");
    println!("    {:<38}{}", "create-func, crt-func", "Create a custom function with a return value allocated at execution");
    println!("    {:<38}{}", "view", "see certain information like the breakpoints that have been placed etc");
    println!("    {:<38}{}", "help, h", "Display this help message");
    println!("\nFor detailed usage, just type <command> without its arguments");
}
