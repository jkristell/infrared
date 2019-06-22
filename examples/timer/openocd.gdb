target extended-remote :3333

# print demangled symbols
set print asm-demangle on

# Quit without confirmation
set confirm off

# detect unhandled exceptions, hard faults and panics
#break DefaultHandler
#break UserHardFault
#break rust_begin_unwind

monitor arm semihosting enable
load
continue
