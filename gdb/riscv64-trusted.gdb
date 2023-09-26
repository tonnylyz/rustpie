target extended-remote 127.0.0.1:1234
add-symbol-file target/riscv64virt/release/rustpi
file trusted/target/riscv64/release/trusted

set confirm off
display/i $pc
