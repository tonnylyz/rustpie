//#[cfg(target_arch = "aarch64")]
pub use self::aarch64::*;
#[cfg(target_arch = "riscv64")]
pub use self::riscv64::*;

//#[cfg(target_arch = "aarch64")]
mod aarch64;

#[cfg(target_arch = "riscv64")]
mod riscv64;

pub trait Address {
  fn va2pa(&self) -> usize;
}

impl Address for usize {
  // #[cfg(target_arch = "aarch64")]
  fn va2pa(&self) -> usize {
    page_table::va2pa(*self).unwrap()
  }
}
