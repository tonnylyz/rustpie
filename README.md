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

1. Nightly Rust (`nightly-2021-06-15-x86_64-unknown-linux-gnu` tested)
2. `rust-src` component (use `make dependencies` to install)
3. QEMU (`6.1.0` tested)
4. LLVM (`llvm-objcopy` and `llvm-objdump`, can be replaced with GNU ones)
5. K210 `kflash` tool [kflash.py](https://github.com/kendryte/kflash.py).
6. `mkimage` u-boot image tool
7. RedoxFS utilities [link](https://gitlab.redox-os.org/redox-os/redoxfs) (`redoxfs` and `redoxfs-mkfs`) Install with `cargo install redoxfs@0.4.4`

## Structure

```
├── Cargo.lock
├── Cargo.toml                  
├── common                      [crate] Information shared between kernel and user
├── gdb                         GDB debug scripts
├── inject                      [crate] Procedural macro for fault injecting
├── lib
│   ├── cs                      [crate] Information shared between client and servers
│   ├── exported                [crate] User program library
│   ├── fs                      [crate] File operation library
│   ├── libtrusted              [crate] Trusted server programming library
│   ├── microcall               [crate] System call library
│   ├── redox                   [crate] Redox types library
│   └── unwind                  [crate] Unwind library
├── Makefile                    Top-level makefile
├── src                         Kernel source
│   ├── arch                    Architecture-related codes
│   ├── board                   Board and platforms definition
│   ├── driver                  Kernel drivers
│   ├── lib                     Kernel library
│   ├── logger.rs               Logger
│   ├── main.rs                 Kernel main
│   ├── misc.rs                 Misc
│   ├── mm                      Memory management
│   ├── panic.rs                Panic handling
│   ├── syscall                 System call
│   └── util.rs                 Utilities
├── trusted                     [crate] Trusted server image
│   ├── build.rs
│   ├── Cargo.lock
│   ├── Cargo.toml
│   ├── Makefile
│   └── src
│       ├── blk                 Block server
│       │   ├── k210_sdcard.rs  K210 SD-card server
│       │   ├── ramdisk.rs      Ramdisk server
│       │   └── virtio_blk.rs   VirtIO-blk server
│       ├── fs                  Ported RedoxFs server
│       ├── main.rs             Trusted main
│       ├── mm.rs               Memory management server
│       ├── panic.rs            Panic handling
│       ├── pm.rs               Process management server
│       ├── root.rs             Root task
│       ├── rtc.rs              Realtime-Clock server
│       ├── terminal.rs         STDIO server
└── user                        [crate] User programs
```

