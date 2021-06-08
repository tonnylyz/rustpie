use spin::{Once, Mutex};
use common::*;
use microcall::mem_alloc;
use super::EntryLike;

static VALLOC_BASE: Once<Mutex<usize>> = Once::new();

pub fn valloc(num_of_page: usize) -> *mut u8 {
  let mut base = match VALLOC_BASE.get() {
    None => { VALLOC_BASE.call_once(|| Mutex::new(0x4_0000_0000)); VALLOC_BASE.get().unwrap() }
    Some(_) => { VALLOC_BASE.get().unwrap() }
  }.lock();

  let current = *base;
  *base += num_of_page * PAGE_SIZE;
  for va in (current..(current + num_of_page * PAGE_SIZE)).step_by(PAGE_SIZE) {
    mem_alloc(0, va, super::Entry::default().attribute());
  }
  current as *mut u8
}
