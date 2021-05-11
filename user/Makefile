
ARCH ?= aarch64
USER_PROFILE ?= debug

ifeq (${USER_PROFILE}, release)
CARGO_FLAGS = --release
else
CARGO_FLAGS =
endif

USER_IMAGE := target/${ARCH}/${USER_PROFILE}/rustpi-user

.PHONY: all clean ${USER_IMAGE}

${USER_IMAGE}:
	cargo build --target targets/${ARCH}.json -Z build-std=core,alloc ${CARGO_FLAGS}

clean:
	-cargo clean
