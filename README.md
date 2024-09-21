# LisaDbg
it's a debugger written in rust
it has a small cli interface but the vast majority of available commands are in the context of the app
# here is the cli interface :

```
LisaDbg 1.7.0

USAGE:
    lisa-dbg [OPTIONS] [--] [file]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --arg <arg>                          set arguments for script to debug
        --attach <attach>                    attach the dbg of a existing process with here pid or here name
        --b-ret <b-ret>...                   to place a breakpoint at ret addr of the function which contain the rva
        --b-ret-va <b-ret-va>...             to place a breakpoint at ret addr of the function which contain the va
        --b-va <b-va>...                     to place a breakpoint at an address (VA) you must know in advance the
                                             address going and
    -b, --breakpoint <breakpoint-addr>...    to place a breakpoint at an address (RVA)
        --exec <exec-cmd>...                 to execute a cmd specified before running dbg
    -w, --watchpoint <watchpts>...           Set a watchpoint in the format '[--memory=<zone>] [--access=<rights>]
                                             <offset>

ARGS:
    <file>
```

As you can see, the options here are limited, 

here are all the commands available before debugging begins (you can access them by typing "help" in the entry at the beginning):
# before dbg
```
LisaDbg Help:
Available commands:
    breakpoint, b                         Sets a breakpoint at the specified address (rva) or symbol
    file                                  Change the current file context
    run                                   Start or resume execution of the debugged program
    reset                                 Reset the debugger settings or context
    remove                                removes a specified element, for more information type "help remove"
    quit, q, exit                         Exit the debugger
    s, sym, symbol                        Load symbols, this will allow commands like "b-ret" to be used with the function name directly
    b-ret                                 places a breakpoint at the return address
    skip                                  skip calls to the specified function
    proc-addr                             get the address of a function in a dll
    hook, ho                              Setup a function hook to redirect execution flow
    create-func, crt-func                 Create a custom function with a return value allocated at execution
    view                                  see certain information like the symbol that have been placed etc
    watchpoint, watch, w                  Set an observation point to a memory location, if the memory location is on the stack, this must be specified
    sym-info                              displays all information of the specified symbol
    arg, args, argv                       defined the arguments with which the debugger will launch the target program
    attach                                to attach the debugger to a running process
    break-va, b-va                        Sets a breakpoint at the specified address (va)
    break-ret-va, b-ret-va                Sets a breakpoint at the ret address of function of addr specified (va)
    def                                   to declare a function or a type or a structure
    help-c                                to display the commands available when the program reaches a breakpoint
    help, h                               Display this help message

For detailed usage, just type help <command> or <command> without its arguments
```

you should know that here, these are not all the debugger options, just the pre-debugging options,

if you put a breakpoint at an address and the dbg stops, you will be able to execute all these commands :
# in dbg
```

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


for more information (if available) just type <command> without its arguments
```
# WARNING
If you are using dwarf symbols and want to monitor local (on the stack) variables, you need to ensure that the "fbreg" field is calculated from the value of rsp before entering the current function
