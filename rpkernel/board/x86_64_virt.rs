use core::ptr::NonNull;

use acpi::{
  madt::{Madt, MadtEntry},
  AcpiHandler, AcpiTables, PhysicalMapping,
};
use rpabi::{
  platform::{Device, PlatformInfo},
  X64BootData,
};
use spin::Once;
use x86_64::instructions::port::Port;

use crate::{
  arch::PAGE_SIZE,
  kernel::{
    interrupt::InterruptController,
    traits::Address,
  },
};

static CPU_NUMBER: Once<usize> = Once::new();
pub fn cpu_number() -> usize {
  *CPU_NUMBER.get().unwrap()
}

pub fn init_per_core() {
  crate::driver::timer::init();
  if crate::core_id() != 0 {
    crate::driver::INTERRUPT_CONTROLLER.init();
  } else {
    // APIC init is done in init_post_heap_setup
  }
}

pub fn core_id() -> usize {
  // TODO
  0
}

pub fn launch_other_cores() {
  // NOTE: starting AP is too complicated. 
  // Let's wait for OVMF implementation of multiprocessor wakeup ACPI 6.4 spec, so we can have an easy method to launch those cores
  // the following is the failed attempt to fabricate a start page
  // const SECOND_START_PPN: u8 = (0x6000 >> PAGE_SHIFT) as u8;
  // extern "C" {
  //   fn secondary_start();
  // }
  // let lapic = crate::driver::apic::local_apic();
  // let secondary_start_low = (((SECOND_START_PPN as usize) << PAGE_SHIFT) | NON_CACHE_BASE as usize) as *mut u8;
  // let secondary_start_high = secondary_start as usize as *mut u8;

  // info!("secondary l {:?}", secondary_start_low);
  // info!("secondary h {:?}", secondary_start_high);
  // unsafe {
  //   memcpy(secondary_start_low, secondary_start_high, PAGE_SIZE);
  //   for i in 0..MAX_CPU_NUMBER {
  //     if i == lapic.id() as usize {
  //       continue;
  //     }
  //     let dest = (i as u32) << 24;
  //     lapic.send_init_ipi(dest);
  //     mdelay(10);
  //     lapic.send_sipi(SECOND_START_PPN, dest);
  //     udelay(200);
  //     lapic.send_sipi(SECOND_START_PPN, dest);
  //   }
  // }
}

pub static PLATFORM_INFO: Once<PlatformInfo> = Once::new();

pub struct I8250 {}

pub static DEBUG_UART: Once<I8250> = Once::initialized(I8250 {});

impl crate::kernel::print::DebugUart for I8250 {
  fn init(&self) {}

  fn putc(&self, c: u8) {
    const COM1: u16 = 0x3f8;
    unsafe {
      let mut thr_port = Port::new(COM1 + 0);
      let mut lsr_port = Port::new(COM1 + 5);
      loop {
        let lsr: u8 = lsr_port.read();
        // if thr is empty, ready to send
        if lsr & (1 << 5) != 0 {
          break;
        }
      }
      thr_port.write(c);
    }
  }

  fn getc(&self) -> Option<u8> {
    const COM1: u16 = 0x3f8;
    unsafe {
      let mut rhr_port = Port::new(COM1 + 0);
      let mut lsr_port = Port::new(COM1 + 5);
      let lsr: u8 = lsr_port.read();
      // if ready flag is set
      if lsr & 0b1 != 0 {
        return Some(rhr_port.read());
      }
    }
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

  PLATFORM_INFO.call_once(|| {
    let mut r = PlatformInfo::default();
    r.devices[0] = Some(Device {
      name: [0; rpabi::platform::DEVICE_NAME_LEN],
      register: 0..0,
      interrupt: None,
      driver: Some(rpabi::platform::Driver::Ramdisk),
    });
    r
  });
  // TODO: we are not ready to boot APs
  CPU_NUMBER.call_once(|| 1);
  (heap_start..heap_end, paged_start..paged_end)
}

pub fn init_post_heap_setup(boot_data: usize) {
  let boot_data = unsafe {
    ((boot_data.pa2kva()) as *const X64BootData)
      .as_ref()
      .unwrap()
  };

  // all CPU use the same page directory before switching to ring-3
  // only setup non cache map on bsp CPU
  crate::arch::mmu::map_non_cache_region_boot();

  let handler = AcpiHandlerImpl;
  unsafe {
    let acpi_table = AcpiTables::from_rsdp(handler, boot_data.acpi2_table_pa).unwrap();
    // Parsing MADT multiple APIC
    let madt = acpi_table.find_table::<Madt>().unwrap();
    for e in madt.entries() {
      // info!("{:?}", e);
      if let MadtEntry::MultiprocessorWakeup(e) = e {
        info!("{:?}", e);
      }
    }
    let platform_info = acpi_table.platform_info().unwrap();

    if let acpi::InterruptModel::Apic(apic) = platform_info.interrupt_model {
      let local_apic_pa = apic.local_apic_address;
      info!("local apic pa {:X}", local_apic_pa);
      for io_apic in apic.io_apics.iter() {
        info!("io apic pa {:X}", io_apic.address);
      }
    } else {
      panic!();
    }
    let processor_info = platform_info.processor_info.unwrap();
    let ap_number = processor_info.application_processors.len();
    info!(
      "found {} application processors, total {} CPUs",
      ap_number,
      ap_number + 1
    );
    let mut i = 1;
    for ap_proc in processor_info.application_processors.iter() {
      info!("ap proc#{} local APIC id {}", i, ap_proc.local_apic_id);
      i += 1;
    }

    // Disable 8259A interrupt controllers (PIC)
    Port::<u8>::new(0x21).write(0xff);
    Port::<u8>::new(0xA1).write(0xff);

    // Setup APIC
    crate::driver::INTERRUPT_CONTROLLER.init();
  };
}

#[derive(Debug, Clone, Copy)]
struct AcpiHandlerImpl;

impl AcpiHandler for AcpiHandlerImpl {
  unsafe fn map_physical_region<T>(
    &self,
    physical_address: usize,
    size: usize,
  ) -> PhysicalMapping<Self, T> {
    PhysicalMapping::new(
      physical_address,
      NonNull::new(physical_address.pa2kva() as *mut _).unwrap(),
      size,
      size,
      Self,
    )
  }

  fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {
    // do nothing
  }
}
