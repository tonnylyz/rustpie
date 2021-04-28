use riscv::{asm::*, regs::*};

use crate::arch::Address;
use core::mem::size_of;

pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
pub const MACHINE_SIZE: usize = size_of::<usize>();

const PA2KVA: usize = 0xFFFF_FFFF_0000_0000;
const KVA2PA: usize = 0xFFFF_FFFF;

impl Address for usize {
  fn pa2kva(&self) -> usize {
    *self | PA2KVA
  }
  fn kva2pa(&self) -> usize {
    *self & KVA2PA
  }
}

pub type Arch = Riscv64Arch;

pub type ContextFrame = super::context_frame::Riscv64ContextFrame;

pub type PageTable = super::page_table::Riscv64PageTable;

pub type ArchPageTableEntry = super::page_table::Riscv64PageTableEntry;

pub type AddressSpaceId = u16;

pub type CoreId = usize;

pub struct Riscv64Arch;

impl crate::arch::ArchTrait for Riscv64Arch {
  fn exception_init() {
    super::exception::init();
  }

  fn invalidate_tlb() {
    riscv::barrier::sfence_vma_all();
  }

  fn wait_for_interrupt() {
    wfi();
  }

  fn nop() {
    nop();
  }

  fn fault_address() -> usize {
    STVAL.get() as usize
  }

  fn core_id() -> CoreId {
    // Note: a pointer to hart_id is stored in sscratch
    unsafe {
      ((SSCRATCH.get() as usize) as *const usize).read()
    }
  }
}