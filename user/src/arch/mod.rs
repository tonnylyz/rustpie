#[cfg(target_arch = "aarch64")]
pub use self::aarch64::*;
#[cfg(target_arch = "riscv64")]
pub use self::riscv64::*;

#[cfg(target_arch = "aarch64")]
mod aarch64;

#[cfg(target_arch = "riscv64")]
mod riscv64;

#[cfg(target_arch = "aarch64")]
const PA2KVA: usize = 0xFFFF_FF80_0000_0000;
#[cfg(target_arch = "aarch64")]
const KVA2PA: usize = 0x0000_007F_FFFF_FFFF;

#[cfg(target_arch = "riscv64")]
const PA2KVA: usize = 0xFFFF_FFFF_0000_0000;
#[cfg(target_arch = "riscv64")]
const KVA2PA: usize = 0xFFFF_FFFF;

pub trait Address {
  fn va2pa(&self) -> usize;
}

impl Address for usize {
  // #[cfg(target_arch = "aarch64")]
  fn va2pa(&self) -> usize {
    page_table::va2pa(*self).unwrap()
  }
}
