.PHONY: all aarch64-pi3-emu aarch64-virt-emu riscv64-virt-emu clean dependencies aarch64_pi3 aarch64_virt

all: aarch64_pi3.bin aarch64_virt.bin riscv64_virt.bin

user/aarch64.elf:
	make -C user aarch64.elf

user/riscv64.elf:
	make -C user riscv64.elf

%.bin: user/aarch64.elf user/riscv64.elf
	RUSTFLAGS="-C llvm-args=-global-isel=false" cargo build --target src/targets/$(basename $@).json --features $(basename $@) -Z build-std=core,alloc
	rust-objcopy target/$(basename $@)/debug/rustpi -O binary $@

aarch64_pi3:
	make -B aarch64_pi3.bin

aarch64-pi3-emu: aarch64_pi3
	qemu-system-aarch64 -M raspi3 -kernel $<.bin -serial null -serial stdio -display none

aarch64_virt:
	make -B aarch64_virt.bin

aarch64-virt-emu: aarch64_virt
	qemu-system-aarch64 -M virt -cpu cortex-a53 -smp 4 -m 4096 -kernel $<.bin -serial stdio -display none

riscv64-virt-emu: riscv64_virt.bin
	qemu-system-riscv64 -M virt -smp 4 -m 1024 -bios default -device loader,file=$<,addr=0x80200000 -serial stdio -display none

clean:
	-cargo clean
	-rm *.bin
	-make -C user clean

dependencies:
	rustup component add rust-src
	rustup component add llvm-tools-preview
	cargo install cargo-binutils