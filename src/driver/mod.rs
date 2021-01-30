#[cfg(feature = "aarch64_virt")]
mod aarch64_virt;
#[cfg(feature = "aarch64_virt")]
pub use self::aarch64_virt::*;

#[cfg(feature = "riscv64_virt")]
mod riscv64_virt;
#[cfg(feature = "riscv64_virt")]
pub use self::riscv64_virt::*;

pub mod mmio;
