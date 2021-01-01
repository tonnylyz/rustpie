#[cfg(any(feature = "aarch64_pi3", feature = "aarch64_virt"))]
pub use self::aarch64::*;
#[cfg(feature = "riscv64_virt")]
pub use self::riscv64::*;

pub use self::traits::*;

#[cfg(any(feature = "aarch64_pi3", feature = "aarch64_virt"))]
mod aarch64;
#[cfg(feature = "riscv64_virt")]
mod riscv64;
mod traits;
