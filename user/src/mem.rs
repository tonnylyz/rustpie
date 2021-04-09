use spin::{Once, Mutex, Spin};
use crate::config::PAGE_SIZE;
use crate::syscall::mem_alloc;
use crate::arch::page_table::PTE_DEFAULT;

static VALLOC_BASE: Once<Mutex<usize>> = Once::new();

pub fn valloc(num_of_page: usize) -> *mut u8 {
  let mut base = match VALLOC_BASE.get() {
    None => { VALLOC_BASE.call_once(|| Mutex::new(0x4_0000_0000)); VALLOC_BASE.get().unwrap() }
    Some(_) => { VALLOC_BASE.get().unwrap() }
  }.lock();

  let current = *base;
  *base += num_of_page * PAGE_SIZE;
  for va in (current..(current + num_of_page * PAGE_SIZE)).step_by(PAGE_SIZE) {
    mem_alloc(0, va, PTE_DEFAULT);
  }
  current as *mut u8
}

pub fn vfree(ptr: *mut u8) {
  // do nothing for the time being
}
