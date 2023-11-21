use core::mem::size_of;
use riscv::{asm::*, regs::*};
use tock_registers::interfaces::{Readable, Writeable};

use crate::kernel::traits::*;

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

pub type AddressSpaceId = u16;

pub type CoreId = usize;

pub struct Riscv64Arch;

impl ArchTrait for Riscv64Arch {
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

  fn raw_arch_id() -> usize {
    // hartid is m-mode only
    panic!()
  }

  fn install_user_page_table(base: usize, asid: AddressSpaceId) {
    SATP.write(
      SATP::MODE::Sv39 + SATP::ASID.val(asid as u64) + SATP::PPN.val((base >> PAGE_SHIFT) as u64),
    );
    riscv::barrier::sfence_vma_all();
  }
}