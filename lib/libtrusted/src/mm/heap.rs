use buddy_system_allocator::LockedHeapWithRescue;
use microcall::mem_alloc;
use common::PAGE_SIZE;
use crate::mm::{Entry, EntryLike};
use core::sync::atomic::{AtomicUsize, Ordering};
use core::alloc::Layout;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeapWithRescue<32> = LockedHeapWithRescue::new(enlarge);

static HEAP_TOP: AtomicUsize = AtomicUsize::new(0);

fn enlarge(heap: &mut buddy_system_allocator::Heap<32>, _layout: &Layout) {
  const HEAP_DELTA_SIZE: usize = 16;
  let delta = HEAP_TOP.fetch_add(PAGE_SIZE * HEAP_DELTA_SIZE, Ordering::Relaxed);
  for i in 0..HEAP_DELTA_SIZE {
    mem_alloc(0, delta + i * PAGE_SIZE, Entry::default().attribute());
  }
  // info!("Enlarge heap {:x} ~ {:x}", delta, delta + PAGE_SIZE * HEAP_DELTA_SIZE);
  unsafe { heap.add_to_heap(delta, delta + PAGE_SIZE * HEAP_DELTA_SIZE); }
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
