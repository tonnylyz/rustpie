use crate::arch::*;
use crate::kernel::traits::*;
use crate::mm::page_table::{Entry, Error, PageTableTrait};
use crate::mm::PhysicalFrame;
use alloc::vec::Vec;
use rpabi::syscall::mm::EntryAttribute;
use x86_64::structures::paging::mapper::PageTableFrameMapping;
use x86_64::structures::paging::{
  FrameAllocator, MappedPageTable, OffsetPageTable, PhysFrame, Size4KiB, Translate,
};
use x86_64::structures::paging::{Mapper, PageTable as PT, PageTableFlags, Size1GiB};
use x86_64::{PhysAddr, VirtAddr};

#[derive(Debug)]
pub struct X64PageTable {
  directory_kva: usize,
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct X64PageTableEntry(usize);

impl X64PageTableEntry {
  fn flags(&self) -> PageTableFlags {
    PageTableFlags::from_bits_retain(self.0 as u64)
  }
}

impl X64PageTableEntry {
  fn to_pa(&self) -> usize {
    self.0 & 0x0000_FFFF_FFFF_F000
  }
}

impl core::convert::From<X64PageTableEntry> for Entry {
  fn from(value: X64PageTableEntry) -> Self {
    let flags = value.flags();
    Entry::new(
      EntryAttribute::new(
        flags.contains(PageTableFlags::WRITABLE),
        flags.contains(PageTableFlags::USER_ACCESSIBLE),
        flags.contains(PageTableFlags::NO_CACHE),
        true, // need no-execute page protection feature
        true, // need no-execute page protection feature
        flags.contains(PageTableFlags::BIT_9),
        flags.contains(PageTableFlags::BIT_10),
      ),
      value.to_pa(),
    )
  }
}

impl core::convert::From<Entry> for X64PageTableEntry {
  fn from(value: Entry) -> Self {
    let mut flags = PageTableFlags::PRESENT;
    if value.attribute().u_shared() {
      flags |= PageTableFlags::BIT_10;
    }
    if value.attribute().copy_on_write() {
      flags |= PageTableFlags::BIT_9;
    }
    if value.attribute().device() {
      flags |= PageTableFlags::NO_CACHE;
    }
    if value.attribute().writable() {
      flags |= PageTableFlags::WRITABLE;
    }
    if value.attribute().u_readable() {
      flags |= PageTableFlags::USER_ACCESSIBLE;
    }
    X64PageTableEntry(value.pa() | (flags.bits() as usize))
  }
}

impl X64PageTable {
  fn l4pt(&self) -> &'static mut PT {
    let l4pt = unsafe { (self.directory_kva as *mut PT).as_mut().unwrap() };
    l4pt
  }

  fn mapped_pt(&self) -> MappedPageTable<'static, X64PageTableFrameMapping> {
    let pt = self.l4pt();
    unsafe { MappedPageTable::new(pt, X64PageTableFrameMapping {}) }
  }
}

impl PageTableTrait for X64PageTable {
  fn new(directory_kva: usize, table_frames: &mut Vec<PhysicalFrame>) -> Self {
    const PHY_ADDR_MAX: usize = 0x1_0000_0000;
    let r = X64PageTable { directory_kva };
    let pt = r.l4pt();
    let mut offset_pt = unsafe { OffsetPageTable::new(pt, VirtAddr::new(0.pa2kva() as u64)) };
    let start_frame = PhysFrame::<Size1GiB>::containing_address(PhysAddr::new(0));
    let end_frame = PhysFrame::containing_address(PhysAddr::new(PHY_ADDR_MAX as u64));
    for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
      let page = x86_64::structures::paging::Page::containing_address(VirtAddr::new(
        (frame.start_address().as_u64() as usize).pa2kva() as u64,
      ));
      trace!("{:?} -> {:?}", page, frame);
      let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
      unsafe {
        let _ = offset_pt
          .map_to(page, frame, flags, &mut X64FrameAllocator(table_frames))
          .unwrap();
      }
    }
    for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
      let page = x86_64::structures::paging::Page::containing_address(VirtAddr::new(
          mmu::NON_CACHE_BASE + frame.start_address().as_u64()
      ));
      let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
      unsafe {
        let _ = offset_pt
        .map_to(page, frame, flags, &mut X64FrameAllocator(table_frames))
          .unwrap();
      }
    }
    r
  }

  fn map(
    &self,
    va: usize,
    pa: usize,
    attr: EntryAttribute,
    table_frames: &mut Vec<PhysicalFrame>,
  ) -> Result<(), Error> {
    type X64Page = x86_64::structures::paging::Page<Size4KiB>;
    let mut pt = self.mapped_pt();
    let pseudo_entry = ArchPageTableEntry::from(Entry::new(attr, pa));
    let page_flags = PageTableFlags::from_bits_truncate(pseudo_entry.0 as u64);
    let table_flags =
      PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE | PageTableFlags::WRITABLE; // for recursive;
    unsafe {
      let page = X64Page::from_start_address(VirtAddr::new(va as u64)).unwrap();
      let frame = PhysFrame::from_start_address(PhysAddr::new(pa as u64)).unwrap();
      trace!("{:?} -> {:?}", page, frame);
      pt.map_to_with_table_flags(
        page,
        frame,
        page_flags,
        table_flags,
        &mut X64FrameAllocator(table_frames),
      )
      .unwrap()
      .flush()
    }
    Ok(())
  }

  fn unmap(&self, va: usize) {
    type X64Page = x86_64::structures::paging::Page<Size4KiB>;
    let mut pt = self.mapped_pt();
    let (_pf, flush) = pt
      .unmap(X64Page::from_start_address(VirtAddr::new(va as u64)).unwrap())
      .unwrap();
    flush.flush();
  }

  fn lookup_page(&self, va: usize) -> Option<Entry> {
    use x86_64::structures::paging::mapper::TranslateResult;
    let pt = self.mapped_pt();
    let result = pt.translate(VirtAddr::new(va as u64));
    match result {
      TranslateResult::Mapped {
        frame,
        offset: _,
        flags,
      } => Some(Entry::new(
        Entry::from(X64PageTableEntry(flags.bits() as usize)).attribute(),
        frame.start_address().as_u64() as usize,
      )),
      TranslateResult::NotMapped => None,
      TranslateResult::InvalidFrameAddress(_) => None,
    }
  }

  fn recursive_map(&self, va: usize) {
    let recursive_index = va >> (PAGE_SHIFT + 3 * 9);
    let flags = PageTableFlags::PRESENT | PageTableFlags::USER_ACCESSIBLE;
    self.l4pt()[recursive_index].set_addr(PhysAddr::new(self.directory_kva.kva2pa() as u64), flags);
  }
}

#[derive(Debug)]
struct X64FrameAllocator<'a>(&'a mut Vec<PhysicalFrame>);

unsafe impl FrameAllocator<Size4KiB> for X64FrameAllocator<'_> {
  fn allocate_frame(&mut self) -> Option<PhysFrame> {
    match crate::mm::page_pool::page_alloc() {
      Ok(phy_frame) => {
        phy_frame.zero();
        let addr = phy_frame.pa();
        let frame = PhysFrame::containing_address(x86_64::PhysAddr::new(addr as u64));
        trace!("alloc pt frame {:x}", addr);
        self.0.push(phy_frame);
        Some(frame)
      }
      Err(_) => None,
    }
  }
}

struct X64PageTableFrameMapping;

unsafe impl PageTableFrameMapping for X64PageTableFrameMapping {
  fn frame_to_pointer(&self, frame: PhysFrame) -> *mut PT {
    let pa = frame.start_address().as_u64() as usize;
    trace!("deref {:x} -> {:x}", pa, pa.pa2kva());
    pa.pa2kva() as *mut PT
  }
}
