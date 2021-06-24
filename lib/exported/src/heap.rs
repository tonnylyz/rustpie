use buddy_system_allocator::LockedHeap;
use common::PAGE_SIZE;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init() {
  const HEAP_SIZE: usize = 16;
  for i in 0..HEAP_SIZE {
    crate::mm::mem_alloc(common::CONFIG_HEAP_BTM + i * PAGE_SIZE).expect("heap alloc failed");
  }
  unsafe {
    HEAP_ALLOCATOR.lock().init(common::CONFIG_HEAP_BTM, HEAP_SIZE * PAGE_SIZE)
  }
}

#[alloc_error_handler]
fn alloc_error_handler(_: core::alloc::Layout) -> ! {
  panic!("alloc_error_handler: heap panic");
}
