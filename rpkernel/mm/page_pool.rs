use alloc::collections::VecDeque;
use core::ops::Range;

use rpabi::syscall::error::ERROR_OOM;
use spin::{Mutex, Once};

use crate::arch::*;
use crate::kernel::traits::Address;
use crate::mm::PhysicalFrame;

pub type Error = usize;

// struct PPAllocator;
//
// unsafe impl Allocator for PPAllocator {
//   fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
//     Global.allocate(layout)
//   }
//
//   unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
//     Global.deallocate(ptr, layout)
//   }
// }


struct PagePool {
  free: VecDeque<usize>,
}

impl PagePool {
  pub fn init(&mut self, range: Range<usize>) {
    assert_eq!(range.start % PAGE_SIZE, 0);
    assert_eq!(range.end % PAGE_SIZE, 0);
    unsafe { core::ptr::write_bytes(range.start.pa2kva() as *mut u8, 0, range.len()); }
    for pa in range.step_by(PAGE_SIZE) {
      self.free.push_back(pa);
    }
  }

  pub fn allocate(&mut self) -> Result<PhysicalFrame, Error> {
    if let Some(pa) = self.free.pop_front() {
      Ok(PhysicalFrame::new(pa))
    } else {
      Err(ERROR_OOM)
    }
  }

  pub fn free(&mut self, pa: usize) -> Result<(), Error> {
    self.free.push_back(pa);
    Ok(())
  }
}


static PAGE_POOL: Once<Mutex<PagePool>> = Once::new();

fn page_pool() -> &'static Mutex<PagePool> {
  PAGE_POOL.get().unwrap()
}

pub fn init(range: Range<usize>) {
  PAGE_POOL.call_once(|| {Mutex::new(PagePool {
    free: VecDeque::new(),
  })});
  let mut pool = page_pool().lock();
  pool.init(range);
}

pub fn page_alloc() -> Result<PhysicalFrame, Error> {
  let mut pool = page_pool().lock();
  pool.allocate()
}

pub fn page_free(pa: usize) -> Result<(), Error> {
  let mut pool = page_pool().lock();
  pool.free(pa)
}