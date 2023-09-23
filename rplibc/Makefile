ARCH ?= aarch64
RPLIBC_PROFILE ?= release

ifeq (${RPLIBC_PROFILE}, release)
CARGO_FLAGS = --release
else
CARGO_FLAGS =
endif

.PHONY: all clean

all:
	cargo build --target src/cargo_target/${ARCH}.json -Z build-std=core,alloc ${CARGO_FLAGS}

clean:
	-cargo clean
