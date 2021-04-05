use crate::arch::{Address, PAGE_SIZE};
use rlibc::memset;
use alloc::sync::Arc;

#[derive(Debug)]
pub struct PageFrame {
  pa: usize,
}

impl PageFrame {
  pub fn new(pa: usize) -> Self {
    assert_eq!(pa % PAGE_SIZE, 0);
    PageFrame {
      pa
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
      memset(self.kva() as *mut u8, 0, PAGE_SIZE);
    }
  }

  pub fn as_slice<T>(&self) -> &'static [T] {
    unsafe { core::slice::from_raw_parts(self.kva() as *const T, PAGE_SIZE / core::mem::size_of::<T>()) }
  }

  pub fn as_mut_slice<T>(&self) -> &'static mut [T] {
    unsafe { core::slice::from_raw_parts_mut(self.kva() as *mut T, PAGE_SIZE / core::mem::size_of::<T>()) }
  }
}

#[derive(Debug, Clone)]
pub struct UserFrame(Arc<PageFrame>);

impl UserFrame {
  pub fn new(frame: PageFrame) -> Self {
    UserFrame(Arc::new(frame))
  }
  pub fn pa(&self) -> usize { self.0.pa }
}

impl Drop for PageFrame {
  fn drop(&mut self) {
    println!("drop frame {:016x}", self.pa);
  }
}
