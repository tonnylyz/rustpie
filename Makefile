.PHONY: all aarch64-pi3-emu aarch64-virt-emu riscv64-virt-emu clean

all: aarch64_pi3.bin riscv64_virt.bin

user/aarch64.elf:
	make -C user aarch64.elf

user/riscv64.elf:
	make -C user riscv64.elf

%.bin: user/aarch64.elf user/riscv64.elf
	cargo +nightly build --target src/targets/$(basename $@).json --features $(basename $@) -Z build-std=core,alloc --verbose
	rust-objcopy target/$(basename $@)/release/rustpi -O binary $@

aarch64-pi3-emu: aarch64_pi3.bin
	qemu-system-aarch64 -M raspi3 -kernel $< -serial null -serial stdio -display none

aarch64-virt-emu: aarch64_virt.bin
	qemu-system-aarch64 -M virt -smp 4 -m 4096 -kernel $< -serial stdio -display none -net none

riscv64-virt-emu: riscv64_virt.bin
	qemu-system-riscv64 -M virt -smp 4 -m 1024 -bios default -device loader,file=$<,addr=0x80200000 -serial stdio -display none

clean:
	cargo clean
	rm *.bin
	make -C user clean
