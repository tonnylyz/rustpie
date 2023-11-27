use x86_64::{
  structures::paging::{
    FrameAllocator, Mapper, OffsetPageTable, PageTable, PageTableFlags, PhysFrame, Size1GiB,
    Size4KiB,
  },
  PhysAddr, VirtAddr,
};

use crate::kernel::traits::Address;

const PHY_ADDR_MAX: usize = 0x1_0000_0000;

/// Offset for MMIO access
pub const NON_CACHE_BASE: u64 = 0xFFFF_F000_0000_0000;

/// Create non-cache identity mapping during init
///
/// rpefistub creates identity mapping from [0, 0x1_0000_0000) to [0xFFFF_8000_0000_0000, 0xFFFF_8001_0000_0000)
/// 
/// kernel may need non-cache/device access to some MMIO like APIC
/// 
/// this function creates identity mapping from [0, 0x1_0000_0000) to [0xFFFF_F000_0000_0000, 0xFFFF_F001_0000_0000)
/// 
/// Page allocation is done through page_alloc. It only modify the page table pointed by CR3.
pub fn map_non_cache_region_boot() {
  let pt = l4pt();
  let mut offset_pt = unsafe { OffsetPageTable::new(pt, VirtAddr::new(0.pa2kva() as u64)) };
  let start_frame = PhysFrame::<Size1GiB>::containing_address(PhysAddr::new(0));
  let end_frame = PhysFrame::containing_address(PhysAddr::new(PHY_ADDR_MAX as u64));
  for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
    let page = x86_64::structures::paging::Page::containing_address(VirtAddr::new(
        NON_CACHE_BASE + frame.start_address().as_u64()
    ));
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    unsafe {
      let _ = offset_pt
        .map_to(page, frame, flags, &mut BootFrameAllocator)
        .unwrap();
    }
  }
}

// the return ref still points at low-address. It should only be used during init
fn l4pt() -> &'static mut PageTable {
  let p4_table_addr = x86_64::registers::control::Cr3::read()
    .0
    .start_address()
    .as_u64();
  let l4_table = unsafe { &mut *(p4_table_addr as *mut PageTable) };
  l4_table
}

#[derive(Debug)]
struct BootFrameAllocator;

unsafe impl FrameAllocator<Size4KiB> for BootFrameAllocator {
  fn allocate_frame(&mut self) -> Option<PhysFrame> {
    match crate::mm::page_pool::page_alloc() {
      Ok(phy_frame) => {
        phy_frame.zero();
        let addr = phy_frame.pa();
        let frame = PhysFrame::containing_address(x86_64::PhysAddr::new(addr as u64));
        Some(frame)
      }
      Err(_) => None,
    }
  }
}
