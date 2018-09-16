python gdb.COMPLETE_EXPRESSION = gdb.COMPLETE_SYMBOL
target remote :3333

# print demangled symbols by default
set print asm-demangle on

monitor arm semihosting enable

load
step
continue
