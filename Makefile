ARCH ?= aarch64
MACHINE ?= virt
PROFILE ?= release
USER_PROFILE ?= release
TRUSTED_PROFILE ?= release

ifeq (${ARCH}, aarch64)
# enable cfi for aarch64 target experimentally
export RUSTFLAGS := -Cforce-frame-pointers=yes -Cforce-frame-pointers=yes -Cembed-bitcode=yes -Clinker-plugin-lto -Zsanitizer=cfi -Ccodegen-units=1
else
export RUSTFLAGS := -Cforce-frame-pointers=yes
endif

CARGO_FLAGS := ${CARGO_FLAGS} --features ${MACHINE}

ifeq (${PROFILE}, release)
CARGO_FLAGS := ${CARGO_FLAGS} --release
endif

ifeq (${TRUSTED_PROFILE}, release)
CARGO_FLAGS := ${CARGO_FLAGS} --features user_release
endif

KERNEL := target/${ARCH}${MACHINE}/${PROFILE}/rustpi

.PHONY: all emu debug dependencies clean disk trusted_image user_image ramdisk.img rplibc user_c_image

all: ${KERNEL} ${KERNEL}.bin ${KERNEL}.asm

${KERNEL}: trusted_image
	cargo build --target src/target/${ARCH}${MACHINE}.json -Z build-std=core,alloc  ${CARGO_FLAGS}

ifeq (${MACHINE}, tx2)
trusted_image: ramdisk.img
else
trusted_image:
endif
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

tftp: ${KERNEL}.bin
	mkimage -n rustpi -A arm64 -O linux -C none -T kernel -a 0x80080000 -e 0x80080000 -d $< ${KERNEL}.ubi
	scp ${KERNEL} root@192.168.106.153:/tftp
	scp ${KERNEL}.ubi root@192.168.106.153:/tftp
	echo "tftp 0xf0200000 192.168.106.153:rustpi; tftp 0x8a000000 192.168.106.153:rustpi.ubi; bootm start 0x8a000000 - 0x80000000; bootm loados; bootm go" | xclip -selection c

clean:
	-cargo clean
	make -C trusted clean
	make -C user clean

disk: user_image user_c_image
	rm -rf disk
	mkdir disk
	redoxfs disk.img disk
	cp user/target/${ARCH}/${USER_PROFILE}/{shell,cat,ls,mkdir,touch,rm,rd,stat,hello,ps,write} disk/
	cp user-c/hello2 disk/
	sync
	umount disk

sdcard: user_image
	rm -rf sdcard
	mkdir sdcard
	sudo redoxfs-mkfs /dev/sda
	sudo redoxfs /dev/sda sdcard
	sudo cp user/target/${ARCH}/${USER_PROFILE}/{shell,cat,ls,mkdir,touch,rm,rd,stat,hello,ps,write} sdcard/
	sync
	sudo umount sdcard

ramdisk.img: user_image
	rm -rf ramdisk
	mkdir ramdisk
	dd if=/dev/zero of=ramdisk.img bs=1M count=4
	redoxfs-mkfs ramdisk.img
	redoxfs ramdisk.img ramdisk
	cp user/target/${ARCH}/${USER_PROFILE}/{shell,cat,ls,mkdir,touch,rm,rd,stat,hello,ps,write} ramdisk/
	sync
	umount ramdisk

dependencies:
	rustup component add rust-src