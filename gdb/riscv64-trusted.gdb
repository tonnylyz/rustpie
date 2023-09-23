target extended-remote 127.0.0.1:1234
add-symbol-file target/riscv64virt/debug/rustpi
file trusted/target/riscv64/debug/trusted

break trusted_main
set confirm off
display/i $pc
