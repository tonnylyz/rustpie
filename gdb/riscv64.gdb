target extended-remote 127.0.0.1:1234
file target/riscv64/debug/rustpi
add-symbol-file target/riscv64/debug/rustpi -o -0xffffffff00000000
break main
set confirm off
display/i $pc
