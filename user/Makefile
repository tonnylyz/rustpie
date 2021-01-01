.PHONY: all clean

all: aarch64.elf riscv64.elf

%.elf:
	cargo build --target src/targets/$(basename $@).json -Zbuild-std=core,alloc --release
	cp target/$(basename $@)/release/rustpi-user $@

clean:
	cargo clean
	rm *.elf
