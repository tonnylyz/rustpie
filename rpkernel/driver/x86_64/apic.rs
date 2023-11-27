use x2apic::ioapic::IoApic;
use x2apic::lapic::{xapic_base, LocalApic, LocalApicBuilder};
use x86_64::instructions::port::Port;

use spin::{Mutex, Once};

use crate::core_id;
use crate::kernel::interrupt::{
  InterProcessInterrupt as IPI, InterProcessorInterruptController, InterruptController,
};

pub const TIMER_INTERRUPT_NUMBER: u8 = 123;
pub const ERROR_INTERRUPT_NUMBER: u8 = 126;
pub const SPURIOUS_INTERRUPT_NUMBER: u8 = 127;

pub const IRQ_MIN: usize = 0x20;

/// The timer IRQ number.
pub const INT_TIMER: usize = TIMER_INTERRUPT_NUMBER as usize;

const IO_APIC_BASE: usize = 0xFEC0_0000;

static mut LOCAL_APIC: Option<LocalApic> = None;
static mut IS_X2APIC: bool = false;
// static mut IO_APIC: Once<Mutex<IoApic>> = Once::new();

fn phys_to_non_cache_va(pa: usize) -> usize {
  pa | (crate::arch::mmu::NON_CACHE_BASE as usize)
}

pub struct Apic {
  //   local_apic: LocalApic,
  io_apic: Mutex<IoApic>,
}

pub static INTERRUPT_CONTROLLER: Once<Apic> = Once::new();

pub type Interrupt = usize;

impl InterruptController for Once<Apic> {
  fn init(&self) {
    if core_id() != 0 {
      unsafe { local_apic().enable() };
    } else {
      info!("Initialize Local APIC...");
      unsafe {
        // Disable 8259A interrupt controllers
        Port::<u8>::new(0x21).write(0xff);
        Port::<u8>::new(0xA1).write(0xff);
      }

      let mut builder = LocalApicBuilder::new();
      builder
        .timer_vector(TIMER_INTERRUPT_NUMBER as _)
        .error_vector(ERROR_INTERRUPT_NUMBER as _)
        .spurious_vector(SPURIOUS_INTERRUPT_NUMBER as _);

      let mut is_x2apic = false;

      if cpu_has_x2apic() {
        info!("Using x2APIC.");
        is_x2apic = true;
      } else {
        let xapic_base_vaddr = phys_to_non_cache_va(unsafe { xapic_base() } as usize);
        info!(
          "Using xAPIC. paddr at {:x} map to {:x}",
          unsafe { xapic_base() } as usize,
          xapic_base_vaddr
        );
        builder.set_xapic_base(xapic_base_vaddr as u64);
      }

      let mut lapic = builder.build().unwrap();
      unsafe {
        lapic.enable();
        LOCAL_APIC = Some(lapic);
        IS_X2APIC = is_x2apic;
      }

      info!("Initialize IO APIC...");
      let ioapic_paddr = IO_APIC_BASE as usize;
      let ioapic_vaddr = phys_to_non_cache_va(ioapic_paddr);

      info!(
        "Initialize IO APIC paddr at {:x} map to {:x}",
        ioapic_paddr, ioapic_vaddr
      );

      unsafe {
        let mut io_apic = IoApic::new(ioapic_vaddr as u64);
        io_apic.init(0x21);

        let max_entry = io_apic.max_table_entry() + 1;
        info!(
          "IOAPIC id {} v{} has {} entries",
          io_apic.id(),
          io_apic.version(),
          max_entry
        );
        for i in 0..max_entry {
          if i != 2 {
            io_apic.enable_irq(i);
          } else {
            io_apic.disable_irq(i);
          }
          // info!("ioapic table entry [{}]\n{:?}", i, io_apic.table_entry(i));
        }
        self.call_once(|| Apic {
          io_apic: Mutex::new(io_apic),
        });
      }
    }
    info!("InterruptController apic init ok");
  }

  fn enable(&self, int: Interrupt) {
    info!("InterruptController apic enable int {}", int);
    if int < TIMER_INTERRUPT_NUMBER as _ {
      unsafe {
        self.get().unwrap().io_apic.lock().enable_irq(int as u8);
      }
    }
  }

  fn disable(&self, int: Interrupt) {
    info!("InterruptController apic disable int {}", int);
    if int < TIMER_INTERRUPT_NUMBER as _ {
      unsafe {
        self.get().unwrap().io_apic.lock().disable_irq(int as u8);
      }
    }
  }

  fn fetch(&self) -> Option<(Interrupt, usize)> {
    todo!()
  }

  fn finish(&self, _int: Interrupt) {
    unsafe { local_apic().end_of_interrupt() }
  }
}

impl InterProcessorInterruptController for Once<Apic> {
  fn send_to_one(&self, irq: IPI, target: usize) {
    todo!()
  }

  fn send_to_multiple(&self, irq: IPI, target_mask: usize) {
    todo!()
  }
}

pub fn local_apic<'a>() -> &'a mut LocalApic {
  // It's safe as LAPIC is per-cpu.
  unsafe { LOCAL_APIC.as_mut().unwrap() }
}

fn cpu_has_x2apic() -> bool {
  match raw_cpuid::CpuId::new().get_feature_info() {
    Some(finfo) => finfo.has_x2apic(),
    None => false,
  }
}
