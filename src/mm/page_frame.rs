use alloc::sync::Arc;

use crate::arch::PAGE_SIZE;
use crate::lib::traits::*;

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
      core::ptr::write_bytes(self.kva() as *mut u8, 0, PAGE_SIZE);
    }
  }

  #[allow(dead_code)]
  pub fn as_slice<T>(&self) -> &'static [T] {
    unsafe { core::slice::from_raw_parts(self.kva() as *const T, PAGE_SIZE / core::mem::size_of::<T>()) }
  }

  pub fn as_mut_slice<T>(&self) -> &'static mut [T] {
    unsafe { core::slice::from_raw_parts_mut(self.kva() as *mut T, PAGE_SIZE / core::mem::size_of::<T>()) }
  }
}

#[derive(Debug, Clone)]
pub enum UserFrame {
  Memory(Arc<PageFrame>),
  Device(usize),
}

impl UserFrame {
  pub fn new_memory(frame: PageFrame) -> Self {
    UserFrame::Memory(Arc::new(frame))
  }
  pub fn new_device(physical_address: usize) -> Self {
    UserFrame::Device(physical_address)
  }
  pub fn pa(&self) -> usize {
    match self {
      UserFrame::Memory(frame) => { frame.pa }
      UserFrame::Device(pa) => { *pa }
    }
  }
}

impl Drop for PageFrame {
  fn drop(&mut self) {
    println!("drop frame {:016x}", self.pa);
  }
}
