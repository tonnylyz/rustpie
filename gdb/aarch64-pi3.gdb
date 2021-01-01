target remote 127.0.0.1:1234
add-symbol-file target/aarch64-pi3/release/rustpi
add-symbol-file target/aarch64-pi3/release/rustpi -o -0xffffff8000000000
break *0x80000
set confirm off
display/i $pc
