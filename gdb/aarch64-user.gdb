target remote 127.0.0.1:1234
file user/aarch64.elf
add-symbol-file target/aarch64/debug/rustpi
break _start
set confirm off
display/i $pc
