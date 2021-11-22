ARCH ?= aarch64
TRUSTED_PROFILE ?= release

# Panic Inject Function
export PI
# Page Fault Inject Function
export FI

ifeq (${TRUSTED_PROFILE}, release)
CARGO_FLAGS = --release
else
CARGO_FLAGS =
endif

CARGO_FLAGS := ${CARGO_FLAGS} --features ${MACHINE}

TRUSTED_IMAGE := target/${ARCH}/${TRUSTED_PROFILE}/trusted

.PHONY: all clean

all:
	cargo build --target src/target/${ARCH}.json -Z build-std=core,alloc ${CARGO_FLAGS}
	llvm-objdump -d ${TRUSTED_IMAGE} > ${TRUSTED_IMAGE}.asm

clean:
	-cargo clean
