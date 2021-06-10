#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64/mod.rs"]
mod arch;

// #[cfg(target_arch = "riscv64")]
// #[path = "arch/riscv64/mod.rs"]
// mod arch;

mod valloc;
mod heap;
mod page_fault;

pub use heap::init as heap_init;
pub use page_fault::init as page_fault_init;
pub use arch::page_table::Entry;
pub use arch::page_table::traverse;
pub use arch::page_table::query;
pub use valloc::valloc;

pub fn virt_to_phys(va: usize) -> usize {
  match arch::page_table::query(va) {
    None => { 0 }
    Some(pte) => {
      pte.address() | (va & (common::PAGE_SIZE - 1))
    }
  }
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
