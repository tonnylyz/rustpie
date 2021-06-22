ARCH ?= aarch64
USER_PROFILE ?= release

ifeq (${USER_PROFILE}, release)
CARGO_FLAGS = --release
else
CARGO_FLAGS =
endif

USER_IMAGE := target/${ARCH}/${USER_PROFILE}/user

.PHONY: all clean ${USER_IMAGE}

${USER_IMAGE}:
	cargo build --target src/target/${ARCH}.json -Z build-std=core,alloc ${CARGO_FLAGS}
	llvm-objdump -d ${USER_IMAGE} > ${USER_IMAGE}.asm

clean:
	-cargo clean
