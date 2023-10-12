ARCH ?= aarch64
TRUSTED_PROFILE ?= release

ifeq (${TRUSTED_PROFILE}, release)
CARGO_FLAGS = --release
else
CARGO_FLAGS =
endif

CARGO_FLAGS := ${CARGO_FLAGS} --no-default-features --features ${MACHINE}

TRUSTED_IMAGE := target/${ARCH}/${TRUSTED_PROFILE}/trusted

.PHONY: all clean

all:
	cargo build --target cargo_target/${ARCH}.json -Z build-std=core,alloc ${CARGO_FLAGS}
	llvm-objcopy --strip-all ${TRUSTED_IMAGE} ${TRUSTED_IMAGE}.bin
	llvm-objdump -d ${TRUSTED_IMAGE} > ${TRUSTED_IMAGE}.asm

clean:
	-cargo clean
