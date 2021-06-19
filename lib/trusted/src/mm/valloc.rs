use spin::Mutex;
use common::*;
use microcall::mem_alloc;
use super::EntryLike;

static VALLOC_BASE: Mutex<usize> = Mutex::new(0x4_0000_0000);

pub fn valloc(num_of_page: usize) -> *mut u8 {
  let mut base = VALLOC_BASE.lock();

  let current = *base;
  *base += num_of_page * PAGE_SIZE;
  for va in (current..(current + num_of_page * PAGE_SIZE)).step_by(PAGE_SIZE) {
    mem_alloc(0, va, super::Entry::default().attribute());
  }
  current as *mut u8
}

pub fn virtual_page_alloc(num_of_page: usize) -> usize {
  let mut base = VALLOC_BASE.lock();

  let current = *base;
  *base += num_of_page * PAGE_SIZE;
  current
}
