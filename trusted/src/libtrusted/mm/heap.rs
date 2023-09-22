use core::alloc::Layout;
use core::sync::atomic::{AtomicUsize, Ordering};

use buddy_system_allocator::LockedHeapWithRescue;
use rpabi::PAGE_SIZE;
use spin::Mutex;

use rpsyscall::mem_alloc;

use crate::libtrusted::mm::default_page_attribute;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeapWithRescue<32> = LockedHeapWithRescue::new(enlarge);

static HEAP_TOP: AtomicUsize = AtomicUsize::new(0);

fn enlarge(heap: &mut buddy_system_allocator::Heap<32>, _layout: &Layout) {
  const HEAP_DELTA_SIZE: usize = 16;
  let delta = HEAP_TOP.fetch_add(PAGE_SIZE * HEAP_DELTA_SIZE, Ordering::Relaxed);
  for i in 0..HEAP_DELTA_SIZE {
    mem_alloc(0, delta + i * PAGE_SIZE, default_page_attribute()).expect("heap enlarge allocate memory failed");
  }
  // info!("Enlarge heap {:x} ~ {:x}", delta, delta + PAGE_SIZE * HEAP_DELTA_SIZE);
  unsafe { heap.add_to_heap(delta, delta + PAGE_SIZE * HEAP_DELTA_SIZE); }
}

pub fn init() {
  const HEAP_INIT_SIZE: usize = 64;
  for i in 0..HEAP_INIT_SIZE {
    mem_alloc(0, rpabi::CONFIG_HEAP_BTM + i * PAGE_SIZE, default_page_attribute()).expect("heap init allocate mem failed");
  }
  HEAP_TOP.store(rpabi::CONFIG_HEAP_BTM + HEAP_INIT_SIZE * PAGE_SIZE, Ordering::Relaxed);
  unsafe {
    HEAP_ALLOCATOR.lock().init(rpabi::CONFIG_HEAP_BTM, HEAP_INIT_SIZE * PAGE_SIZE);
  }
}

static VIRTUAL_HEAP_BASE: Mutex<usize> = Mutex::new(rpabi::CONFIG_VIRTUAL_HEAP_BTM);

pub fn virtual_alloc(num_of_page: usize, alloc_physical: bool) -> Option<usize> {
  let mut base = VIRTUAL_HEAP_BASE.lock();
  let addr = *base;
  *base += num_of_page * PAGE_SIZE;
  drop(base);
  if alloc_physical {
    for i in 0..num_of_page {
      let r = rpsyscall::mem_alloc(0, addr + i * PAGE_SIZE, default_page_attribute());
      if r.is_err() {
        error!("virtual_alloc mem_alloc failed");
      }
    }
  }
  Some(addr)
}

pub fn virtual_free(va: usize, num_of_page: usize) {
  for i in 0..num_of_page {
    let _ = rpsyscall::mem_unmap(0, va + i * PAGE_SIZE);
  }
}
