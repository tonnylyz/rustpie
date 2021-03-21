use crate::board::*;

use super::interface::PAGE_SHIFT;
use super::interface::PAGE_SIZE;
use super::vm_descriptor::*;

const PHYSICAL_ADDRESS_LIMIT_GB: usize = BOARD_PHYSICAL_ADDRESS_LIMIT >> 30;
const ENTRY_PER_PAGE: usize = PAGE_SIZE / 8;

#[derive(Copy, Clone)]
#[repr(transparent)]
struct BlockDescriptor(u64);

impl BlockDescriptor {
  fn new(output_addr: usize, device: bool) -> BlockDescriptor {
    BlockDescriptor((
      PAGE_DESCRIPTOR::PXN::False
          + PAGE_DESCRIPTOR::OUTPUT_PPN.val((output_addr >> PAGE_SHIFT) as u64)
          + PAGE_DESCRIPTOR::AF::True
          + PAGE_DESCRIPTOR::AP::RW_EL1
          + PAGE_DESCRIPTOR::TYPE::Block
          + PAGE_DESCRIPTOR::VALID::True
          +
          if device {
            PAGE_DESCRIPTOR::AttrIndx::DEVICE + PAGE_DESCRIPTOR::SH::OuterShareable
          } else {
            PAGE_DESCRIPTOR::AttrIndx::NORMAL + PAGE_DESCRIPTOR::SH::InnerShareable
          }
    ).value)
  }
  const fn invalid() -> BlockDescriptor {
    BlockDescriptor(0)
  }
}

#[repr(C)]
#[repr(align(4096))]
struct PageTables {
  lvl1: [BlockDescriptor; ENTRY_PER_PAGE],
}

#[no_mangle]
#[link_section = ".data.kvm"]
static mut KPT: PageTables = PageTables {
  lvl1: [BlockDescriptor(0); ENTRY_PER_PAGE],
};

trait BaseAddr {
  fn base_addr_u64(&self) -> u64;
  fn base_addr_usize(&self) -> usize;
}

impl<T> BaseAddr for T {
  fn base_addr_u64(&self) -> u64 {
    self as *const T as u64
  }
  fn base_addr_usize(&self) -> usize {
    self as *const T as usize
  }
}

#[no_mangle]
#[link_section = ".text.kvm"]
pub unsafe extern "C" fn populate_page_table() {
  const ONE_GIGABYTE: usize = 1 << 30;
  for output_addr in (0..BOARD_PHYSICAL_ADDRESS_LIMIT).step_by(ONE_GIGABYTE) {
    if crate::board::BOARD_NORMAL_MEMORY_RANGE.contains(&output_addr) {
      KPT.lvl1[output_addr >> 30] = BlockDescriptor::new(output_addr, false);
    } else if crate::board::BOARD_DEVICE_MEMORY_RANGE.contains(&output_addr) {
      KPT.lvl1[output_addr >> 30] = BlockDescriptor::new(output_addr, true);
    }
  }
  for output_addr in (BOARD_PHYSICAL_ADDRESS_LIMIT..(512 * ONE_GIGABYTE)).step_by(ONE_GIGABYTE) {
    KPT.lvl1[output_addr >> 30] = BlockDescriptor::new(output_addr, false);
  }
}

#[no_mangle]
#[link_section = ".text.kvm"]
pub unsafe extern "C" fn mmu_init() {
  use cortex_a::regs::*;
  use cortex_a::*;
  MAIR_EL1.write(
    MAIR_EL1::Attr0_Normal_Outer::WriteBack_NonTransient_ReadWriteAlloc
        + MAIR_EL1::Attr0_Normal_Inner::WriteBack_NonTransient_ReadWriteAlloc
        + MAIR_EL1::Attr1_Device::nonGathering_nonReordering_noEarlyWriteAck
  );
  TTBR0_EL1.set(KPT.lvl1.base_addr_u64());
  TTBR1_EL1.set(KPT.lvl1.base_addr_u64());

  TCR_EL1.write(TCR_EL1::TBI0::Ignored
      + TCR_EL1::TBI1::Ignored
      + TCR_EL1::IPS.val(0b001) // 64GB
      + TCR_EL1::TG0::KiB_4
      + TCR_EL1::TG1::KiB_4
      + TCR_EL1::SH0::Inner
      + TCR_EL1::SH1::Inner
      + TCR_EL1::ORGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
      + TCR_EL1::ORGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
      + TCR_EL1::IRGN0::WriteBack_ReadAlloc_WriteAlloc_Cacheable
      + TCR_EL1::IRGN1::WriteBack_ReadAlloc_WriteAlloc_Cacheable
      + TCR_EL1::EPD0::EnableTTBR0Walks
      + TCR_EL1::EPD1::EnableTTBR1Walks
      + TCR_EL1::T0SZ.val(64 - 39)
      + TCR_EL1::T1SZ.val(64 - 39));

  barrier::isb(barrier::SY);
  SCTLR_EL1.modify(SCTLR_EL1::M::Enable + SCTLR_EL1::C::Cacheable + SCTLR_EL1::I::Cacheable);
  barrier::isb(barrier::SY);
}
