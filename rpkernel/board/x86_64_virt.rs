use rpabi::{
  platform::PlatformInfo,
  X64BootData,
};
use spin::Once;
use x86_64::instructions::port::Port;

use crate::{
  arch::PAGE_SIZE,
  kernel::traits::Address,
};

static CPU_NUMBER: Once<usize> = Once::new();
pub fn cpu_number() -> usize {
  *CPU_NUMBER.get().unwrap()
}

pub fn init_per_core() {}

pub fn core_id() -> usize {
  0
}

pub fn launch_other_cores() {}

pub static PLATFORM_INFO: Once<PlatformInfo> = Once::new();

pub struct I8250 {}

pub static DEBUG_UART: Once<I8250> = Once::initialized(I8250 {});

impl crate::kernel::print::DebugUart for I8250 {
  fn init(&self) {}

  fn putc(&self, c: u8) {
    unsafe {
      const COM1: u16 = 0x3f8;
      // todo: poll LSR (COM1+5)
      Port::new(COM1 + 0).write(c);
    }
  }

  fn getc(&self) -> Option<u8> {
    None
  }
}

pub fn init(boot_data: usize) -> (core::ops::Range<usize>, core::ops::Range<usize>) {
  let boot_data = unsafe {
    ((boot_data.pa2kva()) as *const X64BootData)
      .as_ref()
      .unwrap()
  };
  let free_mib = boot_data.free_mem_count * PAGE_SIZE / 1024 / 1024;
  println!("Free memory {} MiB", free_mib);

  extern "C" {
    fn KERNEL_END();
  }

  let kernel_end = crate::util::round_up((KERNEL_END as usize).kva2pa(), PAGE_SIZE);
  let heap_start = kernel_end;
  let heap_end = heap_start + crate::mm::config::CONFIG_NON_PAGED_MEMORY_SIZE;
  let paged_start = heap_end;
  let paged_end = boot_data.free_mem_start + boot_data.free_mem_count * PAGE_SIZE;

  CPU_NUMBER.call_once(|| 1);
  (heap_start..heap_end, paged_start..paged_end)
}
