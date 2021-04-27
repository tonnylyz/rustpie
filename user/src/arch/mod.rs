#[cfg(target_arch = "aarch64")]
pub use self::aarch64::*;
#[cfg(target_arch = "riscv64")]
pub use self::riscv64::*;


#[cfg(target_arch = "aarch64")]
mod aarch64;

#[cfg(target_arch = "riscv64")]
mod riscv64;


pub trait Address {
  fn va2pa(&self) -> usize;
}

pub trait EntryLike {
  fn executable(&self) -> bool;
  fn writable(&self) -> bool;
  fn copy_on_write(&self) -> bool;
  fn shared(&self) -> bool;

  fn set_executable(&mut self, b: bool);
  fn set_writable(&mut self, b: bool);
  fn set_copy_on_write(&mut self, b: bool);
  fn set_shared(&mut self, b: bool);

  fn address(&self) -> usize;
  fn set_address(&mut self, addr: usize);

  fn attribute(&self) -> usize;
  fn set_attribute(&mut self, attr: usize);

  fn is_valid(&self) -> bool;
  fn is_table(&self) -> bool;
  fn is_page(&self) -> bool;
}

use crate::config::PAGE_SIZE;

impl Address for usize {
  // #[cfg(target_arch = "aarch64")]
  fn va2pa(&self) -> usize {
    match self::page_table::query(*self) {
      None => { 0 }
      Some(pte) => {
        pte.address() | (*self & (PAGE_SIZE - 1))
      }
    }
  }
}
