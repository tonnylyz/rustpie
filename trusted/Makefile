ARCH ?= aarch64
TRUSTED_PROFILE ?= release

ifeq (${TRUSTED_PROFILE}, release)
CARGO_FLAGS = --release
else
CARGO_FLAGS =
endif

TRUSTED_IMAGE := target/${ARCH}/${TRUSTED_PROFILE}/trusted

.PHONY: all clean ${TRUSTED_IMAGE}

${TRUSTED_IMAGE}:
	cargo build --target src/target/${ARCH}.json -Z build-std=core,alloc ${CARGO_FLAGS}
	llvm-objdump -d ${TRUSTED_IMAGE} > ${TRUSTED_IMAGE}.asm

clean:
	-cargo clean
