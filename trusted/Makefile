ARCH ?= aarch64
TRUSTED_PROFILE ?= release

ifeq (${TRUSTED_PROFILE}, release)
CARGO_FLAGS = --release
else
CARGO_FLAGS =
endif

ifeq (${MACHINE}, k210)
CARGO_FLAGS := ${CARGO_FLAGS} --features k210
endif

TRUSTED_IMAGE := target/${TARGET}/${TRUSTED_PROFILE}/trusted

.PHONY: all clean

all:
	cargo build --target cargo_target/${TARGET}.json -Z build-std=core,alloc ${CARGO_FLAGS}
	llvm-objcopy --strip-all ${TRUSTED_IMAGE} ${TRUSTED_IMAGE}.bin
	llvm-objdump -d ${TRUSTED_IMAGE} > ${TRUSTED_IMAGE}.asm

clean:
	-cargo clean
