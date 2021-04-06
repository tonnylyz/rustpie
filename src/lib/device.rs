use alloc::vec::Vec;
use core::ops::Range;

use crate::arch::PAGE_SIZE;
use crate::driver::Interrupt;
use crate::lib::round_down;
use crate::mm::UserFrame;

#[derive(Debug)]
pub struct Device {
  pub registers: Vec<Range<usize>>,
  pub interrupts: Vec<Interrupt>,
}

impl Device {
  pub fn new(registers: Vec<Range<usize>>,
             interrupts: Vec<Interrupt>) -> Self {
    Device{
      registers,
      interrupts
    }
  }

  pub fn to_user_frames(&self) -> Vec<UserFrame> {
    let mut result = Vec::new();
    for range in self.registers.iter() {
      let start = round_down(range.start, PAGE_SIZE);
      for pa in (start..range.end).step_by(PAGE_SIZE) {
        result.push(UserFrame::new_device(pa));
      }
    }
    result
  }
}
