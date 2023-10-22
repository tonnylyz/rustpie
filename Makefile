ARCH ?= aarch64
MACHINE ?= virt
PROFILE ?= release
USER_PROFILE ?= release
TRUSTED_PROFILE ?= release
GIC_VERSION ?= 3
# NOTE: generate frame pointer for every function
export RUSTFLAGS := ${RUSTFLAGS} -C force-frame-pointers=yes

CARGO_FLAGS := ${CARGO_FLAGS} --no-default-features --features ${MACHINE}

ifeq (${PROFILE}, release)
CARGO_FLAGS := ${CARGO_FLAGS} --release
endif

ifeq (${TRUSTED_PROFILE}, release)
CARGO_FLAGS := ${CARGO_FLAGS} --features user_release
endif

ifeq (${GIC_VERSION}, 3)
CARGO_FLAGS := ${CARGO_FLAGS} --features gicv3
endif

KERNEL := target/${ARCH}${MACHINE}/${PROFILE}/rustpi

.PHONY: all emu debug dependencies clean disk trusted_image user_image rplibc user_c_image

all: ${KERNEL} ${KERNEL}.bin ${KERNEL}.asm

${KERNEL}: trusted_image
	cargo build --target rpkernel/cargo_target/${ARCH}${MACHINE}.json -Z build-std=core,alloc  ${CARGO_FLAGS}

trusted_image:
	make ARCH=${ARCH} TRUSTED_PROFILE=${TRUSTED_PROFILE} MACHINE=${MACHINE} -C trusted

user_image:
	make ARCH=${ARCH} USER_PROFILE=${USER_PROFILE} -C user

rplibc:
	make ARCH=${ARCH} -C rplibc

user_c_image: rplibc
	make ARCH=${ARCH} -C user-c

${KERNEL}.bin: ${KERNEL}
	llvm-objcopy $< -O binary $@

${KERNEL}-flash.bin: ${KERNEL}.bin
	cat rustsbi-k210.bin ${KERNEL}.bin > ${KERNEL}-flash.bin

${KERNEL}.asm: ${KERNEL}
	llvm-objdump --demangle -d $< > $@

ifeq (${ARCH}, aarch64)
ifeq (${GIC_VERSION}, 3)
QEMU_CMD := qemu-system-aarch64 -M virt,gic-version=3,its=off -cpu cortex-a53 -device loader,file=${KERNEL},addr=0x80000000,force-raw=on
endif
ifeq (${GIC_VERSION}, 2)
QEMU_CMD := qemu-system-aarch64 -M virt -cpu cortex-a53 -device loader,file=${KERNEL},addr=0x80000000,force-raw=on
endif
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

disk: user_image user_c_image
	dd if=/dev/zero of=disk.img bs=1M count=1024
	redoxfs-mkfs disk.img
	rm -rf disk
	mkdir disk
	redoxfs disk.img disk
	# @trap 'umount disk' EXIT
	for f in shell cat ls mkdir touch rm rd stat hello ps write date; do cp user/target/${ARCH}/${USER_PROFILE}/$$f disk/; done
	cp user-c/hello2 disk/
	sync
	umount disk

# sdcard: user_image
# 	rm -rf sdcard
# 	mkdir sdcard
# 	sudo redoxfs-mkfs /dev/sda
# 	sudo redoxfs /dev/sda sdcard
# 	sudo cp user/target/${ARCH}/${USER_PROFILE}/{shell,cat,ls,mkdir,touch,rm,rd,stat,hello,ps,write} sdcard/
# 	sync
# 	sudo umount sdcard

dependencies:
	rustup component add rust-src