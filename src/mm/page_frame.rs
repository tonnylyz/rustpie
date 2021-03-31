use crate::arch::{Address, PAGE_SIZE};

#[derive(Clone, Copy, Debug)]
pub struct PageFrame {
  pa: usize,
}

impl PageFrame {
  pub fn new(pa: usize) -> Self {
    assert_eq!(pa % PAGE_SIZE, 0);
    PageFrame {
      pa,
    }
  }
  pub fn kva(&self) -> usize {
    self.pa.pa2kva()
  }
  pub fn pa(&self) -> usize {
    self.pa
  }
  pub fn zero(&self) {
    unsafe {
      core::intrinsics::volatile_set_memory(self.kva() as *mut u8, 0, PAGE_SIZE);
    }
  }
  #[allow(dead_code)]
  pub fn copy_to(&self, dest: &PageFrame) {
    unsafe {
      core::intrinsics::volatile_copy_memory(dest.kva() as *mut u8, self.kva() as *mut u8, PAGE_SIZE);
    }
  }
  #[allow(dead_code)]
  pub fn copy_from(&self, src: &PageFrame) {
    unsafe {
      core::intrinsics::volatile_copy_memory(self.kva() as *mut u8, src.kva() as *mut u8, PAGE_SIZE);
    }
  }
  pub fn as_slice(&self) -> &'static [u8] {
    unsafe { core::slice::from_raw_parts(self.kva() as *const u8, 4096) }
  }
  pub fn as_mut_slice(&self) -> &'static mut [u8] {
    unsafe { core::slice::from_raw_parts_mut(self.kva() as *mut u8, 4096) }
  }
}
