use core::mem::size_of;

use crate::kernel::traits::*;

pub const PAGE_SHIFT: usize = 12;
pub const PAGE_SIZE: usize = 1 << PAGE_SHIFT;
pub const MACHINE_SIZE: usize = size_of::<usize>();

const PA2KVA: usize = 0xFFFF_8000_0000_0000;
const KVA2PA: usize = 0x0000_007F_FFFF_FFFF;

impl Address for usize {
  fn pa2kva(&self) -> usize {
    *self | PA2KVA
  }
  fn kva2pa(&self) -> usize {
    *self & KVA2PA
  }
}

pub type Arch = X64Arch;

pub type ContextFrame = super::context_frame::X64ContextFrame;

pub type PageTable = super::page_table::X64PageTable;

pub type ArchPageTableEntry = super::page_table::X64PageTableEntry;

pub type AddressSpaceId = u16;

pub type CoreId = usize;

pub struct X64Arch;

impl ArchTrait for X64Arch {
  fn exception_init() {
    super::exception::init();
  }

  fn invalidate_tlb() {
    x86_64::instructions::tlb::flush_all()
  }

  fn wait_for_interrupt() {
    x86_64::instructions::hlt()
  }

  fn nop() {
    x86_64::instructions::nop()
  }

  fn fault_address() -> usize {
    x86_64::registers::control::Cr2::read_raw() as usize
  }

  fn raw_arch_id() -> usize {
    todo!()
  }
}