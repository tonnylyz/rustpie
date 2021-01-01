#[cfg(feature = "aarch64_pi3")]
mod aarch64_pi3;
#[cfg(feature = "aarch64_pi3")]
pub use self::aarch64_pi3::*;

#[cfg(feature = "aarch64_virt")]
mod aarch64_virt;
#[cfg(feature = "aarch64_virt")]
pub use self::aarch64_virt::*;

#[cfg(feature = "riscv64_virt")]
mod riscv64_virt;
#[cfg(feature = "riscv64_virt")]
pub use self::riscv64_virt::*;


#[cfg(all(feature = "aarch64_pi3", feature = "aarch64_virt"))]
compile_error!("features `aarch64_pi3` and `aarch64_virt` are mutually exclusive");

#[cfg(all(feature = "aarch64_pi3", feature = "riscv64_virt"))]
compile_error!("features `aarch64_pi3` and `riscv64_virt` are mutually exclusive");

#[cfg(all(feature = "aarch64_virt", feature = "riscv64_virt"))]
compile_error!("features `aarch64_virt` and `riscv64_virt` are mutually exclusive");