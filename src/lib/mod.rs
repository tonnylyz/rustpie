use crate::arch::CoreTrait;
use crate::lib::core::Core;

pub mod print;
pub mod isr;
pub mod address_space;
pub mod elf;
pub mod user_image;
pub mod scheduler;
pub mod syscall;
pub mod page_table;
pub mod thread;
pub mod bitmap;
pub mod core;
pub mod interrupt;
pub mod itc;

#[inline(always)]
pub fn round_up(addr: usize, n: usize) -> usize {
  (addr + n - 1) & !(n - 1)
}

#[inline(always)]
pub fn round_down(addr: usize, n: usize) -> usize {
  addr & !(n - 1)
}

pub fn current_core() -> &'static Core {
  crate::lib::core::current()
}

#[inline(always)]
pub fn current_thread() -> Option<self::thread::Thread> {
  let core = crate::lib::core::current();
  core.running_thread()
}

#[inline(always)]
pub fn current_process() -> Option<self::address_space::AddressSpace> {
  match current_thread() {
    None => { None }
    Some(t) => { t.address_space() }
  }
}
