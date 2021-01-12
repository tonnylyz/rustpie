target remote 127.0.0.1:1234
file target/aarch64_pi3/debug/rustpi
add-symbol-file target/aarch64_pi3/debug/rustpi -o -0xffffff8000000000
break *0x80000
set confirm off
display/i $pc
