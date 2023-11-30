# Rustpi Enhanced

Rustpi is a research micro-kernel written in Rust.

## Boards and Platforms

Rustpi now supports following platforms.

| MACHINE | ARCH                    | Description                             |
|---------|-------------------------|-----------------------------------------|
| virt    | **aarch64** (AArch64)   | QEMU virt machine (qemu-system-aarch64) |
| virt    | **riscv64** (RISC-V 64) | QEMU virt machine (qemu-system-riscv64) |
| virt    | **x86_64**  (x64) NEW!  | QEMU virt machine (qemu-system-x86_64)  |


For QEMU target, use this line to build and emulate:
```
make MACHINE=virt ARCH=[aarch64|riscv64|x86_64] emu
```

Note: K210 machine is no longer maintained, and will be removed in the future. If you want it, please checkout previous commit.

## Toolchains

1. Nightly Rust (building is tested with latest nightly toolchain with GitHub actions)
2. `rust-src` component installed by `rustup component add rust-src`
3. QEMU (`8.0.2` tested)
4. LLVM tools (`llvm-objcopy` and `llvm-objdump`, also feel free to use GNU ones)
5. LLVM lds (`ld.lld` for C compatibility)
6. RedoxFS utilities [link](https://gitlab.redox-os.org/redox-os/redoxfs) (`redoxfs` and `redoxfs-mkfs`) Install with `cargo install redoxfs@0.4.4` (requires host fuse dev package)

## Host dependencies

Ubuntu:
```
sudo apt install llvm lld libfuse-dev
```
Archlinux:
```
sudo pacman -Sy clang llvm lld fuse2 qemu-system-aarch64 qemu-system-riscv qemu-system-x86
```

