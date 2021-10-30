use alloc::sync::Arc;

use crate::arch::PAGE_SIZE;
use crate::lib::traits::*;

#[derive(Debug)]
pub struct PhysicalFrame {
  pa: usize,
}

impl PhysicalFrame {
  pub fn new(pa: usize) -> Self {
    assert_eq!(pa % PAGE_SIZE, 0);
    PhysicalFrame {
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

  pub fn as_slice<T>(&self) -> &'static [T] {
    unsafe { core::slice::from_raw_parts(self.kva() as *const T, PAGE_SIZE / core::mem::size_of::<T>()) }
  }

  pub fn as_mut_slice<T>(&self) -> &'static mut [T] {
    unsafe { core::slice::from_raw_parts_mut(self.kva() as *mut T, PAGE_SIZE / core::mem::size_of::<T>()) }
  }
}

#[derive(Debug, Clone)]
pub enum Frame {
  PhysicalMemory(Arc<PhysicalFrame>),
  Device(usize),
}

impl From<PhysicalFrame> for Frame {
  fn from(physical_frame: PhysicalFrame) -> Self {
    Frame::PhysicalMemory(Arc::new(physical_frame))
  }
}

impl From<usize> for Frame {
  fn from(device_pa: usize) -> Self {
    Frame::Device(device_pa)
  }
}

impl Frame {
  pub fn pa(&self) -> usize {
    match self {
      Frame::PhysicalMemory(frame) => { frame.pa }
      Frame::Device(pa) => { *pa }
    }
  }
}

impl Drop for PhysicalFrame {
  fn drop(&mut self) {
    trace!("drop frame {:016x}", self.pa);
    super::page_pool::page_free(self.pa).expect("physical page drop failed");
  }
}
