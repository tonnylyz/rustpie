use buddy_system_allocator::LockedHeap;
use microcall::mem_alloc;
use common::PAGE_SIZE;
use crate::mm::{Entry, EntryLike};

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init() {
  const HEAP_SIZE: usize = 16;
  for i in 0..HEAP_SIZE {
    mem_alloc(0, common::CONFIG_HEAP_BTM + i * PAGE_SIZE, Entry::default().attribute());
  }
  unsafe {
    HEAP_ALLOCATOR.lock().init(common::CONFIG_HEAP_BTM, HEAP_SIZE * PAGE_SIZE)
  }
}

#[alloc_error_handler]
fn alloc_error_handler(_: core::alloc::Layout) -> ! {
  panic!("alloc_error_handler: heap panic");
}
