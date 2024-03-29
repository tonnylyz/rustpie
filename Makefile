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

ifeq (${ARCH}, aarch64)
ifeq (${GIC_VERSION}, 3)
CARGO_FLAGS := ${CARGO_FLAGS} --features gicv3
endif
endif

ifeq (${ARCH}, aarch64)
KERNEL_TARGET := aarch64-virt-rustpi
TRUSTED_TARGET := aarch64-unknown-rustpi
USER_TARGET := aarch64-unknown-rustpi
endif
ifeq (${ARCH}, riscv64)
ifeq (${MACHINE}, virt)
KERNEL_TARGET := riscv64gc-virt-rustpi-elf
endif
ifeq (${MACHINE}, k210)
KERNEL_TARGET := riscv64gc-k210-rustpi-elf
endif
TRUSTED_TARGET := riscv64gc-unknown-rustpi-elf
USER_TARGET := riscv64gc-unknown-rustpi-elf
endif
ifeq (${ARCH}, x86_64)
KERNEL_TARGET := x86_64-virt-rustpi
TRUSTED_TARGET := x86_64-unknown-rustpi
USER_TARGET := x86_64-unknown-rustpi
BIOS_DIR ?= /usr/share/ovmf/x64/OVMF.fd
endif

KERNEL := target/${KERNEL_TARGET}/${PROFILE}/rustpi

.PHONY: all emu debug dependencies clean disk trusted_image user_image rplibc user_c_image force

ifeq (${ARCH}, x86_64)
EFISTUB := rpefistub/target/x86_64-unknown-uefi/release/rpefistub.efi

all: ${KERNEL} ${KERNEL}.bin ${KERNEL}.asm ${KERNEL}.sec ${EFISTUB} user_image rplibc user_c_image

${EFISTUB}: ${KERNEL}.bin force
	make ARCH=${ARCH} -C rpefistub

else
all: ${KERNEL} ${KERNEL}.bin ${KERNEL}.asm ${KERNEL}.sec user_image rplibc user_c_image
endif

${KERNEL}: trusted_image force
	cargo build --target rpkernel/cargo_target/${KERNEL_TARGET}.json -Z build-std=core,alloc  ${CARGO_FLAGS}

ifeq (${ARCH}, x86_64)
trusted_image: ramdisk.img
else
trusted_image:
endif
	make ARCH=${ARCH} TRUSTED_PROFILE=${TRUSTED_PROFILE} MACHINE=${MACHINE} TARGET=${TRUSTED_TARGET} -C trusted

user_image:
	make ARCH=${ARCH} USER_PROFILE=${USER_PROFILE} TARGET=${USER_TARGET} -C user

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

${KERNEL}.sec: ${KERNEL}
	llvm-readelf -S $< > $@

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

ifeq (${ARCH}, x86_64)
QEMU_CMD := qemu-system-x86_64 -bios ${BIOS_DIR}
QEMU_DISK_OPTIONS := 
QEMU_COMMON_OPTIONS := -serial stdio -display none -smp 4 -m 2048

emu: ${EFISTUB} ${KERNEL}.asm
	${QEMU_CMD} ${QEMU_COMMON_OPTIONS} ${QEMU_DISK_OPTIONS} -kernel $< -s

debug: ${EFISTUB} ${KERNEL}.asm
	${QEMU_CMD} ${QEMU_COMMON_OPTIONS} ${QEMU_DISK_OPTIONS} -kernel $< -s -S

else
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

endif

clean:
	-cargo clean
	make -C trusted clean
	make -C user clean

disk: user_image user_c_image
	test -f disk.img || (dd if=/dev/zero of=disk.img bs=1M count=1024 && redoxfs-mkfs disk.img)
	true || (mountpoint -q disk && umount disk)
	rm -rf disk
	mkdir disk
	redoxfs disk.img disk
	for f in shell cat ls mkdir touch rm rd stat hello ps write date; do cp user/target/${USER_TARGET}/${USER_PROFILE}/$$f disk; done
	cp user-c/hello2 disk
	sync
	umount disk

ramdisk.img: user_image user_c_image
	test -f $@ || (dd if=/dev/zero of=$@ bs=1M count=4 && redoxfs-mkfs $@)
	true || (mountpoint -q disk && umount disk)
	rm -rf ramdisk
	mkdir ramdisk
	redoxfs $@ ramdisk
	for f in shell cat ls mkdir touch rm rd stat hello ps write date; do cp user/target/${USER_TARGET}/${USER_PROFILE}/$$f ramdisk; done
	cp user-c/hello2 ramdisk
	sync
	umount ramdisk

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