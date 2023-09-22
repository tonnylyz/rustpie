use buddy_system_allocator::LockedHeap;
use rpabi::PAGE_SIZE;

const HEAP_SIZE: usize = 16;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::empty();

pub fn init() {
  for i in 0..HEAP_SIZE {
    crate::mm::page_alloc(rpabi::CONFIG_HEAP_BTM + i * PAGE_SIZE).expect("heap alloc failed");
  }
  unsafe {
    HEAP_ALLOCATOR.lock().init(rpabi::CONFIG_HEAP_BTM, HEAP_SIZE * PAGE_SIZE)
  }
}
