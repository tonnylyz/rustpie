use core::mem::size_of;

use aarch64_cpu::registers::*;
use tock_registers::interfaces::{Readable, Writeable};

use crate::kernel::traits::*;

pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
pub const MACHINE_SIZE: usize = size_of::<usize>();

const PA2KVA: usize = 0xFFFF_FF80_0000_0000;
const KVA2PA: usize = 0x0000_007F_FFFF_FFFF;

impl Address for usize {
  fn pa2kva(&self) -> usize {
    *self | PA2KVA
  }
  fn kva2pa(&self) -> usize {
    *self & KVA2PA
  }
}

pub type Arch = Aarch64Arch;

pub type ContextFrame = super::context_frame::Aarch64ContextFrame;

pub type PageTable = super::page_table::Aarch64PageTable;

pub type AddressSpaceId = u16;

pub type CoreId = usize;

pub struct Aarch64Arch;

impl ArchTrait for Aarch64Arch {
  fn exception_init() {
    super::exception::init();
  }

  fn invalidate_tlb() {
    unsafe {
      core::arch::asm!("dsb ishst");
      core::arch::asm!("tlbi vmalle1is");
      core::arch::asm!("dsb ish");
      core::arch::asm!("isb");
    }
  }

  fn wait_for_interrupt() {
    aarch64_cpu::asm::wfi();
  }

  fn nop() {
    aarch64_cpu::asm::nop();
  }

  fn fault_address() -> usize {
    FAR_EL1.get() as usize
  }

  fn raw_arch_id() -> usize {
    MPIDR_EL1.get() as usize
  }
  
  fn install_user_page_table(base: usize, _asid: AddressSpaceId) {
    use aarch64_cpu::registers::TTBR0_EL1;
    TTBR0_EL1.write(TTBR0_EL1::BADDR.val((base >> 1) as u64));
    Self::invalidate_tlb();
  }
}