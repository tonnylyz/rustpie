#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64.rs"]
mod page_table;

#[cfg(target_arch = "riscv64")]
#[path = "arch/riscv64.rs"]
mod page_table;
mod heap;

pub use heap::init as heap_init;
pub use page_table::Entry;
pub use page_table::traverse;
pub use page_table::query;

use spin::Mutex;
use common::*;
use microcall::mem_alloc;

pub fn default_page_attribute() -> usize {
  Entry::new(true, true, false, false).attribute()
}

pub fn virt_to_phys(va: usize) -> usize {
  match page_table::query(va) {
    None => { 0 }
    Some(pte) => {
      pte.address() | (va & (common::PAGE_SIZE - 1))
    }
  }
}

static VALLOC_BASE: Mutex<usize> = Mutex::new(0x4_0000_0000);

pub fn valloc(num_of_page: usize) -> *mut u8 {
  let mut base = VALLOC_BASE.lock();

  let current = *base;
  *base += num_of_page * PAGE_SIZE;
  for va in (current..(current + num_of_page * PAGE_SIZE)).step_by(PAGE_SIZE) {
    mem_alloc(0, va, default_page_attribute());
  }
  current as *mut u8
}

pub fn virtual_page_alloc(num_of_page: usize) -> usize {
  let mut base = VALLOC_BASE.lock();

  let current = *base;
  *base += num_of_page * PAGE_SIZE;
  current
}

pub trait PageAttribute {
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
