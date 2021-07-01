use buddy_system_allocator::{LockedHeap, LockedHeapWithRescue};
use microcall::mem_alloc;
use common::PAGE_SIZE;
use crate::mm::{Entry, EntryLike};
use core::sync::atomic::{AtomicUsize, Ordering};

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeapWithRescue = LockedHeapWithRescue::new(enlarge);

static HEAP_TOP: AtomicUsize = AtomicUsize::new(0);

fn enlarge(heap: &mut buddy_system_allocator::Heap) {
  let delta = HEAP_TOP.fetch_add(PAGE_SIZE, Ordering::Relaxed);
  mem_alloc(0, delta, Entry::default().attribute());
  unsafe { heap.add_to_heap(delta, delta + PAGE_SIZE); }
}

pub fn init() {
  const HEAP_INIT_SIZE: usize = 16;
  for i in 0..HEAP_INIT_SIZE {
    mem_alloc(0, common::CONFIG_HEAP_BTM + i * PAGE_SIZE, Entry::default().attribute());
  }
  HEAP_TOP.store(common::CONFIG_HEAP_BTM + HEAP_INIT_SIZE * PAGE_SIZE, Ordering::Relaxed);
  unsafe {
    HEAP_ALLOCATOR.lock().init(common::CONFIG_HEAP_BTM, HEAP_INIT_SIZE * PAGE_SIZE)
  }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
  panic!("alloc_error_handler: heap panic {:?}", layout);
}
