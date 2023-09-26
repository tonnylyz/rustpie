target extended-remote 127.0.0.1:1234
file target/riscv64virt/release/rustpi
# add-symbol-file target/riscv64virt/release/rustpi -o -0xffffffff00000000

break pop_context_first
set confirm off
display/i $pc
