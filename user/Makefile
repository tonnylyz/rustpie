.PHONY: all clean

all: aarch64.elf riscv64.elf

%.elf:
	RUSTFLAGS="-C llvm-args=-global-isel=false" \
	cargo build --target targets/$(basename $@).json -Z build-std=core,alloc --release
	cp target/$(basename $@)/release/rustpi-user $@

clean:
	-cargo clean
	-rm *.elf
