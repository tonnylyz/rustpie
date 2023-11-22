ARCH ?= aarch64

.PHONY: all clean

CC := clang
LD := ld.lld

ifeq (${ARCH}, aarch64)
TRIPLE := aarch64-arm-none-eabi
endif
ifeq (${ARCH}, riscv64)
TRIPLE := riscv64-unknown-elf
endif
ifeq (${ARCH}, x86_64)
TRIPLE := x86_64-unknown-none
endif

all:
	${CC} -o hello.o hello.c -I../rplibc/src -target ${TRIPLE} -c
	${LD} -L../rplibc/target/${ARCH}/release -lrplibc -o hello2 --gc-sections -Tlinker.ld       hello.o

clean:
	-rm hello2 hello.o
