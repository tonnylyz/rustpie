use alloc::vec::Vec;
use core::ops::Range;

use crate::arch::PAGE_SIZE;
use crate::driver::Interrupt;
use crate::util::round_down;
use crate::mm::UserFrame;
use crate::lib::event::Event;
use crate::lib::cpu::CoreTrait;

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

pub fn interrupt(int: Interrupt) {
  println!("[IRQ] external {}", int);
  match crate::lib::interrupt::INTERRUPT_WAIT.get(int) {
    None => { println!("[IRQ] irq not registered"); }
    Some(t) => {
      match t.address_space() {
        None => { panic!("kernel thread interrupt?") }
        Some(a) => {
          match a.event_handler(Event::Interrupt(int)) {
            None => { println!("[IRQ] no event handler") }
            Some((pc, sp)) => {
              let nt = crate::lib::thread::new_user(pc, sp, int, a.clone(), None);
              nt.set_status(crate::lib::thread::Status::TsRunnable);

              crate::driver::timer::next();
              crate::lib::cpu::current().schedule();
            }
          }
        }
      }
    }
  }
}