CARGO_FLAGS = --release
export RUSTFLAGS := ${RUSTFLAGS} -C link-args=/debug:dwarf
KERNEL_STUB_TARGET := x86_64-unknown-uefi

.PHONY: all clean

all:
	cargo build --target ${KERNEL_STUB_TARGET} -Z build-std=core,alloc ${CARGO_FLAGS}

clean:
	-cargo clean
