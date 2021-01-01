// rCore buddy system allocator
use buddy_system_allocator::LockedHeap;

use crate::config::{HEAP_BTM, PAGE_SIZE};
use crate::arch::page_table::PTE_DEFAULT;
use crate::syscall::mem_alloc;

#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init() {
  match mem_alloc(0, HEAP_BTM, PTE_DEFAULT) {
    Ok(_) => {}
    Err(_) => { panic!("heap: init: mem_alloc failed") }
  }
  unsafe {
    HEAP_ALLOCATOR.lock().init(HEAP_BTM, PAGE_SIZE)
  }
}

#[alloc_error_handler]
fn alloc_error_handler(_: core::alloc::Layout) -> ! {
  panic!("alloc_error_handler: heap panic");
}