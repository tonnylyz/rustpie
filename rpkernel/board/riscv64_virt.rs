use core::ops::Range;
use core::sync::atomic::{AtomicBool, Ordering};
use hardware::ns16550::*;
use riscv::regs::SSCRATCH;
use spin::Once;
use tock_registers::interfaces::{Readable, Writeable};

use crate::kernel::device::{device_from_fdt_node, PlatformInfo};
use crate::kernel::interrupt::InterruptController;
use crate::kernel::print::DebugUart;
use crate::kernel::traits::*;
use crate::MAX_CPU_NUMBER;

static CPU_NUMBER: spin::Once<usize> = spin::Once::new();
pub fn cpu_number() -> usize {
  *CPU_NUMBER.get().unwrap()
}

pub fn init(fdt: usize) -> (Range<usize>, Range<usize>) {
  // println!("FDT phyaddr {:x}", fdt);
  let fdt = unsafe { core::slice::from_raw_parts(fdt.pa2kva() as *const u8, 8192) };
  let fdt = fdt::Fdt::new(fdt).unwrap();
  // println!("FDT model {}", fdt.root().model());
  // println!("FDT compatible {}", fdt.root().compatible().first());

  CPU_NUMBER.call_once(|| fdt.cpus().count());

  extern "C" {
    // Note: link-time label, see linker.ld
    fn KERNEL_END();
  }
  use crate::arch::PAGE_SIZE;
  let kernel_end = crate::util::round_up((KERNEL_END as usize).kva2pa(), PAGE_SIZE);
  let heap_start = kernel_end;
  let heap_end = heap_start + crate::mm::config::CONFIG_NON_PAGED_MEMORY_SIZE;
  let paged_start = heap_end;
  assert_eq!(kernel_end % PAGE_SIZE, 0);
  assert_eq!(heap_start % PAGE_SIZE, 0);
  assert_eq!(heap_end % PAGE_SIZE, 0);

  let memory_node = fdt.memory();
  let mut region_iter = memory_node.regions();
  let first_region = region_iter
    .next()
    .expect("require at least one memory region");
  let memory_start = first_region.starting_address as usize;
  let memory_range = memory_start..(memory_start + first_region.size.unwrap());

  assert!(memory_range.contains(&kernel_end));
  assert!(memory_range.contains(&heap_start));
  assert!(memory_range.contains(&heap_end));
  assert!(memory_range.contains(&paged_start));
  let paged_end = memory_range.end;

  let chosen = fdt.chosen();
  if let Some(stdout) = chosen.stdout() {
    let range = stdout.node().reg().unwrap().next().unwrap();
    let start_addr = range.starting_address as usize;
    DEBUG_UART
      .call_once(|| Ns16550Mmio::new(start_addr.pa2kva()))
      .init();
  }

  PLATFORM_INFO.call_once(|| {
    let mut r = PlatformInfo::default();
    if let Some(x) = fdt.find_node("/soc/virtio_mmio@10001000") {
      // add first virtio,mmio
      r.devices[0] = Some(device_from_fdt_node(
        &fdt,
        &x,
        Some(rpabi::platform::Driver::VirtioBlk),
      ));
    }
    if let Some(x) = fdt.find_compatible(&["google,goldfish-rtc"]) {
      r.devices[1] = Some(device_from_fdt_node(
        &fdt,
        &x,
        Some(rpabi::platform::Driver::GoldfishRtc),
      ));
    }
    if let Some(x) = fdt.find_compatible(&["ns16550a"]) {
      r.devices[2] = Some(device_from_fdt_node(
        &fdt,
        &x,
        Some(rpabi::platform::Driver::Ns16550),
      ));
    }
    r
  });
  (heap_start..heap_end, paged_start..paged_end)
}

pub fn init_post_heap_setup(fdt: usize) {}

pub fn init_per_core() {
  crate::driver::timer::init();
  crate::driver::INTERRUPT_CONTROLLER.init();
  use riscv::regs::SIE;
  // enable
  //    * timer interrupt
  //    * external interrupt (from PLIC)
  //    * software interrupt (IPI)
  SIE.write(SIE::STIE::SET + SIE::SEIE::SET + SIE::SSIE::SET);
  info!("SIE val {:b}", SIE.get());
}

pub fn core_id() -> usize {
  // Note: a pointer to hart_id is stored in sscratch
  unsafe { ((SSCRATCH.get() as usize) as *const usize).read() }
}

pub fn launch_other_cores() {
  HART_SPIN.store(true, Ordering::Relaxed);
}

static HART_SPIN: AtomicBool = AtomicBool::new(false);

#[no_mangle]
pub unsafe extern "C" fn hart_spin(core_id: usize, fdt: usize) {
  extern "C" {
    fn KERNEL_ENTRY();
  }
  static mut FDT: usize = 0;
  static mut HART_BOOT: usize = 0xffff_ffff;
  if HART_BOOT == 0xffff_ffff {
    HART_BOOT = core_id;
    FDT = fdt;
    for i in 0..MAX_CPU_NUMBER {
      if i != core_id {
        let _ = crate::driver::hsm::hart_start(i, (KERNEL_ENTRY as usize).kva2pa(), 0);
      }
    }
  }

  if core_id == 0 {
    crate::main(core_id, FDT);
  }
  while !HART_SPIN.load(Ordering::Acquire) {}
  crate::main(core_id, FDT);
}

// virtio_mmio@10001000 {
//   interrupts = <0x01>;
//   interrupt-parent = <0x09>;
//   reg = <0x00 0x10001000 0x00 0x1000>;
//   compatible = "virtio,mmio";
// };

// rtc@101000 {
//   interrupts = <0x0b>;
//   interrupt-parent = <0x09>;
//   reg = <0x00 0x101000 0x00 0x1000>;
//   compatible = "google,goldfish-rtc";
// };

// serial@10000000 {
//   interrupts = <0x0a>;
//   interrupt-parent = <0x09>;
//   clock-frequency = "\08@";
//   reg = <0x00 0x10000000 0x00 0x100>;
//   compatible = "ns16550a";
// };

pub static PLATFORM_INFO: Once<PlatformInfo> = Once::new();

pub static DEBUG_UART: Once<Ns16550Mmio> = Once::new();

impl crate::kernel::print::DebugUart for Ns16550Mmio {
  fn init(&self) {
    self.ISR_FCR.write(ISR_FCR::EN_FIFO::Mode16550);
  }

  fn putc(&self, c: u8) {
    fn send(uart: &Ns16550Mmio, c: u8) {
      while !uart.LSR.is_set(LSR::THRE) {
        // Wait until it is possible to write data.
      }
      uart.RHR_THR_DLL.set(c);
    }
    if c == b'\n' {
      send(self, b'\r');
    }
    send(self, c);
  }

  fn getc(&self) -> Option<u8> {
    if self.LSR.is_set(LSR::RDR) {
      Some(self.RHR_THR_DLL.get() as u8)
    } else {
      None
    }
  }
}
