# Rustpi Enhanced

Rustpi is a research micro-kernel. It is also my dissertation project. 
An abstract paper was published at ISSRE 2021 conference.

## Boards and Platforms

Rustpi now supports following platforms.

| MACHINE | ARCH                    | Description                             |
|---------|-------------------------|-----------------------------------------|
| virt    | **aarch64** (AArch64)   | QEMU virt machine (qemu-system-aarch64) |
| virt    | **riscv64** (RISC-V 64) | QEMU virt machine (qemu-system-riscv64) |
| tx2     | **aarch64** (AArch64)   | NVIDIA TX2                              |
| k210    | **riscv64** (RISC-V 64) | Kendryte K210                           |


For QEMU target, use this line to build and emulate:
```
make MACHINE=virt ARCH=[aarch64|riscv64] emu
```

For TX2 target, use this line to build a u-boot image and upload to a TFTP server:
```
make MACHINE=tx2 ARCH=aarch64 tftp
```

For K210 target, use this line to flash:
```
make MACHINE=k210 ARCH=riscv64 flash
```
K210 also require a SBI image. I suggest using [RustSBI](https://github.com/rustsbi/rustsbi/releases/tag/v0.1.1).

## Toolchains

1. Nightly Rust (`rustc 1.72.0-nightly (114fb86ca 2023-06-15)` tested)
2. `rust-src` component (use `make dependencies` to install)
3. QEMU (`8.0.2` tested)
4. LLVM tools (`llvm-objcopy` and `llvm-objdump`, can be replaced with GNU ones)
4. LLVM lds (`ld.lld` for C compatibility)
5. K210 `kflash` tool [kflash.py](https://github.com/kendryte/kflash.py).
6. `mkimage` u-boot image tool
7. RedoxFS utilities [link](https://gitlab.redox-os.org/redox-os/redoxfs) (`redoxfs` and `redoxfs-mkfs`) Install with `cargo install redoxfs@0.4.4`, `libfuse-dev`` is required.

For Ubuntu:
```
sudo apt install llvm lld libfuse-dev
```