use alloc::alloc::Global;
use alloc::vec::Vec;
use core::alloc::{Allocator, AllocError, Layout};
use core::ops::Range;
use core::ptr::NonNull;

use common::syscall::error::{ERROR_INVARG, ERROR_OOM};
use spin::Mutex;

use crate::arch::*;
use crate::mm::PhysicalFrame;

pub type Error = usize;

struct PPAllocator;

unsafe impl Allocator for PPAllocator {
  fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
    Global.allocate(layout)
  }

  unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
    Global.deallocate(ptr, layout)
  }
}


struct PagePool {
  free: Vec<usize, PPAllocator>,
  allocated: Vec<usize, PPAllocator>,
}

impl PagePool {
  pub fn init(&mut self, range: Range<usize>) {
    assert_eq!(range.start % PAGE_SIZE, 0);
    assert_eq!(range.end % PAGE_SIZE, 0);
    for pa in range.step_by(PAGE_SIZE) {
      self.free.push(pa);
    }
  }

  pub fn allocate(&mut self) -> Result<PhysicalFrame, Error> {
    if let Some(pa) = self.free.pop() {
      self.allocated.push(pa);
      Ok(PhysicalFrame::new(pa))
    } else {
      Err(ERROR_OOM)
    }
  }

  pub fn free(&mut self, pa: usize) -> Result<(), Error> {
    if !self.allocated.contains(&pa) {
      Err(ERROR_INVARG)
    } else {
      self.allocated.retain(|p| { *p != pa });
      self.free.push(pa);
      Ok(())
    }
  }
}


static PAGE_POOL: Mutex<PagePool> = Mutex::new(PagePool {
  free: Vec::new_in(PPAllocator),
  allocated: Vec::new_in(PPAllocator),
});

pub fn init() {
  let range = super::config::paged_range();
  let mut pool = PAGE_POOL.lock();
  pool.init(range);
}

pub fn page_alloc() -> Result<PhysicalFrame, Error> {
  let mut pool = PAGE_POOL.lock();
  pool.allocate()
}

pub fn page_free(pa: usize) -> Result<(), Error> {
  let mut pool = PAGE_POOL.lock();
  pool.free(pa)
}