ARCH ?= aarch64
MACHINE ?= virt
PROFILE ?= release
USER_PROFILE ?= release
TRUSTED_PROFILE ?= release

# NOTE: this is to deal with `(signal: 11, SIGSEGV: invalid memory reference)`
# https://github.com/rust-lang/rust/issues/73677
RUSTFLAGS := -C llvm-args=-global-isel=false

# NOTE: generate frame pointer for every function
export RUSTFLAGS := ${RUSTFLAGS} -C force-frame-pointers=yes

ifeq (${MACHINE}, guest)
CARGO_FLAGS := ${CARGO_FLAGS} --features guest
endif

ifeq (${MACHINE}, k210)
CARGO_FLAGS := ${CARGO_FLAGS} --features k210
endif

ifeq (${PROFILE}, release)
CARGO_FLAGS := ${CARGO_FLAGS} --release
endif

ifeq (${TRUSTED_PROFILE}, release)
CARGO_FLAGS := ${CARGO_FLAGS} --features user_release
endif

#TRUSTED_IMAGE := trusted/target/${ARCH}/${TRUSTED_PROFILE}/trusted

ifeq (${MACHINE}, k210)
KERNEL := target/${ARCH}k210/${PROFILE}/rustpi
else
KERNEL := target/${ARCH}/${PROFILE}/rustpi
endif

.PHONY: all emu debug dependencies clean disk trusted_image user_image

all: ${KERNEL} ${KERNEL}.bin ${KERNEL}.asm

${KERNEL}: trusted_image
ifeq (${MACHINE}, k210)
	cargo build --target src/target/${ARCH}k210.json -Z build-std=core,alloc,std ${CARGO_FLAGS}
else
	cargo build --target src/target/${ARCH}.json -Z build-std=core,alloc,std ${CARGO_FLAGS}
endif

trusted_image:
	make ARCH=${ARCH} TRUSTED_PROFILE=${TRUSTED_PROFILE} MACHINE=${MACHINE} -C trusted

user_image:
	make ARCH=${ARCH} USER_PROFILE=${USER_PROFILE} -C user

${KERNEL}.bin: ${KERNEL}
	llvm-objcopy $< -O binary $@

${KERNEL}-flash.bin: ${KERNEL}.bin
	cat rustsbi-k210.bin ${KERNEL}.bin > ${KERNEL}-flash.bin

${KERNEL}.asm: ${KERNEL}
	llvm-objdump --demangle -d $< > $@

ifeq (${ARCH}, aarch64)
QEMU_CMD := qemu-system-aarch64 -M virt -cpu cortex-a53 -device loader,file=${KERNEL},addr=0x80000000,force-raw=on
endif
ifeq (${ARCH}, riscv64)
QEMU_CMD := qemu-system-riscv64 -M virt -bios default -device loader,file=${KERNEL},addr=0xc0000000,force-raw=on
endif

QEMU_DISK_OPTIONS := -drive file=disk.img,if=none,format=raw,id=x0 \
					 -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
					 -global virtio-mmio.force-legacy=false
QEMU_COMMON_OPTIONS := -serial stdio -display none -smp 4 -m 2048

emu: ${KERNEL}.bin ${KERNEL}.asm disk
	${QEMU_CMD} ${QEMU_COMMON_OPTIONS} ${QEMU_DISK_OPTIONS} -kernel $< -s

debug: ${KERNEL}.bin ${KERNEL}.asm disk
	${QEMU_CMD} ${QEMU_COMMON_OPTIONS} ${QEMU_DISK_OPTIONS} -kernel $< -s -S

flash: ${KERNEL}-flash.bin
	sudo kflash -tp /dev/ttyUSB0 -b 3000000 -B dan ${KERNEL}-flash.bin

clean:
	-cargo clean
	make -C trusted clean
	make -C user clean

disk: user_image
	rm -rf disk
	mkdir disk
	redoxfs disk.img disk
	cp user/target/${ARCH}/${USER_PROFILE}/shell disk/
	cp user/target/${ARCH}/${USER_PROFILE}/cat disk/
	cp user/target/${ARCH}/${USER_PROFILE}/ls disk/
	cp user/target/${ARCH}/${USER_PROFILE}/mkdir disk/
	cp user/target/${ARCH}/${USER_PROFILE}/touch disk/
	cp user/target/${ARCH}/${USER_PROFILE}/rm disk/
	cp user/target/${ARCH}/${USER_PROFILE}/rd disk/
	cp user/target/${ARCH}/${USER_PROFILE}/stat disk/
	cp user/target/${ARCH}/${USER_PROFILE}/test disk/
	cp user/target/${ARCH}/${USER_PROFILE}/hello disk/
	cp user/target/${ARCH}/${USER_PROFILE}/ps disk/
	cp user/target/${ARCH}/${USER_PROFILE}/write disk/
	sync
	umount disk

sdcard: user_image
	rm -rf sdcard
	mkdir sdcard
	sudo redoxfs-mkfs /dev/sda
	sudo redoxfs /dev/sda sdcard
	sudo cp user/target/${ARCH}/${USER_PROFILE}/shell sdcard/
	sudo cp user/target/${ARCH}/${USER_PROFILE}/cat sdcard/
	sudo cp user/target/${ARCH}/${USER_PROFILE}/ls sdcard/
	sudo cp user/target/${ARCH}/${USER_PROFILE}/mkdir sdcard/
	sudo cp user/target/${ARCH}/${USER_PROFILE}/touch sdcard/
	sudo cp user/target/${ARCH}/${USER_PROFILE}/rm sdcard/
	sudo cp user/target/${ARCH}/${USER_PROFILE}/rd sdcard/
	sudo cp user/target/${ARCH}/${USER_PROFILE}/stat sdcard/
	sudo cp user/target/${ARCH}/${USER_PROFILE}/test sdcard/
	sudo cp user/target/${ARCH}/${USER_PROFILE}/hello sdcard/
	sudo cp user/target/${ARCH}/${USER_PROFILE}/ps sdcard/
	sudo cp user/target/${ARCH}/${USER_PROFILE}/write sdcard/
	sync
	sudo umount sdcard


dependencies:
	rustup component add rust-src