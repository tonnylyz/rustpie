
ARCH ?= aarch64
PROFILE ?= debug
USER_PROFILE ?= debug

# NOTE: this is to deal with `(signal: 11, SIGSEGV: invalid memory reference)`
# https://github.com/rust-lang/rust/issues/73677
RUSTFLAGS := -C llvm-args=-global-isel=false

# NOTE: generate frame pointer for every function
export RUSTFLAGS := ${RUSTFLAGS} -C force-frame-pointers=yes

ifeq (${PROFILE}, release)
CARGO_FLAGS = --release
endif

ifeq (${USER_PROFILE}, release)
CARGO_FLAGS = --features user_release
endif

USER_IMAGE := user/target/${ARCH}/${USER_PROFILE}/rustpi-user

KERNEL := target/${ARCH}/${PROFILE}/rustpi

.PHONY: all emu debug dependencies ${USER_IMAGE} clean

all: ${KERNEL} ${KERNEL}.bin ${KERNEL}.asm

${KERNEL}: ${USER_IMAGE}
	cargo build --target src/targets/${ARCH}.json --features ${ARCH}_virt -Z build-std=core,alloc ${CARGO_FLAGS}

${USER_IMAGE}:
	make ARCH=${ARCH} USER_PROFILE=${USER_PROFILE} -C user

${KERNEL}.bin: ${KERNEL}
	rust-objcopy $< -O binary $@

${KERNEL}.asm: ${KERNEL}
	rust-objdump -d $< > $@


QEMU_DISK_OPTIONS := -drive file=disk.img,if=none,format=raw,id=x0 \
					 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
                     -global virtio-mmio.force-legacy=false


emu: ${KERNEL}.bin
ifeq (${ARCH}, aarch64)
	qemu-system-aarch64 -M virt,virtualization=on -cpu cortex-a53 -smp 4 -m 2048 -kernel $< -serial stdio -display none \
 		-device loader,file=${KERNEL},addr=0x80000000,force-raw=on \
		${QEMU_DISK_OPTIONS}
else
	qemu-system-riscv64 -M virt -smp 4 -m 2048 -bios default -kernel $< -serial stdio -display none \
		-device loader,file=${KERNEL},addr=0xc0000000,force-raw=on \
		${QEMU_DISK_OPTIONS}
endif

debug: ${KERNEL}.bin
ifeq (${ARCH}, aarch64)
	qemu-system-aarch64 -M virt,virtualization=on -cpu cortex-a53 -smp 4 -m 2048 -kernel $< -serial stdio -display none -s -S \
		-device loader,file=${KERNEL},addr=0x80000000,force-raw=on \
		-${QEMU_DISK_OPTIONS}
else
	qemu-system-riscv64 -M virt -smp 4 -m 2048 -bios default -kernel $< -serial stdio -display none -s -S \
		-device loader,file=${KERNEL},addr=0xc0000000,force-raw=on \
		${QEMU_DISK_OPTIONS}
endif

clean:
	-cargo clean
	make -C user clean

dependencies:
	rustup component add rust-src
	rustup component add llvm-tools-preview
	cargo install cargo-binutils