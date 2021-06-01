// rCore buddy system allocator
use buddy_system_allocator::LockedHeap;
use crate::lib::traits::*;
use core::alloc::{GlobalAlloc, Layout};

pub fn init() {
  // let range = super::config::heap_range();
  // unsafe {
  //   HEAP_ALLOCATOR.lock().init(range.start.pa2kva(), range.end - range.start)
  // }
}

struct Failure;
unsafe impl GlobalAlloc for Failure {
  #[inline(never)]
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    info!("{:?}", layout);
    core::ptr::null_mut()
  }

  unsafe fn dealloc(&self, _: *mut u8, _: Layout) {}
}

// #[global_allocator]
// static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

#[global_allocator]
static HEAP_ALLOCATOR: Failure = Failure;


#[alloc_error_handler]
fn alloc_error_handler(_: Layout) -> ! {
  panic!("alloc_error_handler: heap panic");
}
