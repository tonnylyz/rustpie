target remote 127.0.0.1:1234
#add-symbol-file target/x86_64-unknown-uefi/release/rustpi.efi 0xFFFF80007E56F000 -s .data 0xFFFF80007E576000
add-symbol-file target/x86_64-virt-rustpi/release/rustpi
set confirm off
display/i $pc
set print asm-demangle on
