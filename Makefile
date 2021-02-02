.PHONY: all emu-aarch64 emu-riscv64 debug-aarch64 debug-riscv64 clean dependencies aarch64.bin riscv64.bin

all: aarch64.bin riscv64.bin

user/aarch64.elf:
	make -C user aarch64.elf

aarch64.bin: user/aarch64.elf
	RUSTFLAGS="-C llvm-args=-global-isel=false" \
	cargo build --target src/targets/aarch64.json --features aarch64_virt -Z build-std=core,alloc
	rust-objcopy target/aarch64/debug/rustpi -O binary aarch64.bin

user/riscv64.elf:
	make -C user riscv64.elf

riscv64.bin: user/riscv64.elf
	RUSTFLAGS="-C llvm-args=-global-isel=false" \
	cargo build --target src/targets/riscv64.json --features riscv64_virt -Z build-std=core,alloc
	rust-objcopy target/riscv64/debug/rustpi -O binary riscv64.bin

emu-aarch64: aarch64.bin
	qemu-system-aarch64 -M virt,virtualization=on -cpu cortex-a53 -smp 4 -m 1024 -kernel $< -serial stdio -display none

emu-riscv64: riscv64.bin
	qemu-system-riscv64 -M virt -smp 4 -m 1024 -bios default -kernel $< -serial stdio -display none

debug-aarch64: aarch64.bin
	qemu-system-aarch64 -M virt,virtualization=on -cpu cortex-a53 -smp 4 -m 1024 -kernel $< -serial stdio -display none -s -S

debug-riscv64: riscv64.bin
	qemu-system-riscv64 -M virt -smp 4 -m 1024 -bios default -kernel $< -serial stdio -display none -s -S

clean:
	-cargo clean
	-rm *.bin
	-make -C user clean

dependencies:
	rustup component add rust-src
	rustup component add llvm-tools-preview
	cargo install cargo-binutils