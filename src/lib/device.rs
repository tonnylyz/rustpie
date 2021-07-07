use alloc::vec::Vec;
use core::ops::Range;

use crate::arch::PAGE_SIZE;
use crate::driver::Interrupt;
use crate::mm::Frame;
use crate::util::round_down;

#[derive(Debug)]
pub struct Device {
  pub name: &'static str,
  pub registers: Vec<Range<usize>>,
  pub interrupts: Vec<Interrupt>,
}

impl Device {
  pub fn new(
    name: &'static str, registers: Vec<Range<usize>>,
    interrupts: Vec<Interrupt>) -> Self {
    Device {
      name,
      registers,
      interrupts,
    }
  }

  pub fn to_user_frames(&self) -> Vec<Frame> {
    let mut result = Vec::new();
    for range in self.registers.iter() {
      let start = round_down(range.start, PAGE_SIZE);
      for pa in (start..range.end).step_by(PAGE_SIZE) {
        result.push(Frame::from(pa));
      }
    }
    result
  }
}