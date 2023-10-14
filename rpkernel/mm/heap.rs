use core::ops::Range;

use buddy_system_allocator::LockedHeap;

use crate::kernel::traits::*;

pub fn init(range: Range<usize>) {
  unsafe {
    HEAP_ALLOCATOR.lock().init(range.start.pa2kva(), range.end - range.start)
  }
}

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap<32> = LockedHeap::empty();
