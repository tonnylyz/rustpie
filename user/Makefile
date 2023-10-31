ARCH ?= aarch64
USER_PROFILE ?= release

ifeq (${USER_PROFILE}, release)
CARGO_FLAGS = --release
else
CARGO_FLAGS =
endif

.PHONY: all clean

all:
	cargo build --bins --target src/cargo_target/${TARGET}.json -Z build-std=core,alloc ${CARGO_FLAGS}

clean:
	-cargo clean
