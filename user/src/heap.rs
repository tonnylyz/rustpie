use buddy_system_allocator::LockedHeap;

use crate::config::{HEAP_BTM, PAGE_SIZE};
use crate::syscall::mem_alloc;
use crate::arch::page_table::Entry;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init() {
  const HEAP_SIZE: usize = 16;
  for i in 0..HEAP_SIZE {
    mem_alloc(0, HEAP_BTM + i * PAGE_SIZE, Entry::default());
  }
  unsafe {
    HEAP_ALLOCATOR.lock().init(HEAP_BTM, HEAP_SIZE * PAGE_SIZE)
  }
}

#[alloc_error_handler]
fn alloc_error_handler(_: core::alloc::Layout) -> ! {
  panic!("alloc_error_handler: heap panic");
}
