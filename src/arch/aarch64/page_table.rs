use alloc::vec::Vec;

use spin::Mutex;

use crate::arch::*;
use crate::mm::page_table::{Entry, EntryAttribute, Error, PageTableEntryAttrTrait, PageTableTrait};
use crate::mm::{PhysicalFrame, Frame};

use common::mm::vm_descriptor::*;

use crate::lib::traits::*;
use common::syscall::error::ERROR_INVARG;
use tock_registers::interfaces::Writeable;

pub const PAGE_TABLE_L1_SHIFT: usize = 30;
pub const PAGE_TABLE_L2_SHIFT: usize = 21;
pub const PAGE_TABLE_L3_SHIFT: usize = 12;

#[derive(Debug)]
pub struct Aarch64PageTable {
  directory: PhysicalFrame,
  pages: Mutex<Vec<PhysicalFrame>>,
  user_pages: Mutex<Vec<Frame>>,
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct Aarch64PageTableEntry(usize);

impl ArchPageTableEntryTrait for Aarch64PageTableEntry {
  fn from_pte(value: usize) -> Self {
    Aarch64PageTableEntry(value)
  }

  fn from_pa(pa: usize) -> Self {
    Aarch64PageTableEntry(pa)
  }

  fn to_pte(&self) -> usize {
    self.0
  }

  fn to_pa(&self) -> usize {
    self.0 & 0x0000_FFFF_FFFF_F000
  }

  fn to_kva(&self) -> usize {
    self.to_pa().pa2kva()
  }

  fn valid(&self) -> bool {
    self.0 & 0b11 != 0
  }

  fn entry(&self, index: usize) -> Aarch64PageTableEntry {
    let addr = self.to_kva() + index * MACHINE_SIZE;
    unsafe { Aarch64PageTableEntry((addr as *const usize).read_volatile()) }
  }

  fn set_entry(&self, index: usize, value: Aarch64PageTableEntry) {
    let addr = self.to_kva() + index * MACHINE_SIZE;
    unsafe { (addr as *mut usize).write_volatile(value.0) }
  }

  fn make_table(frame_pa: usize) -> Self {
    Aarch64PageTableEntry::from(Entry::new(EntryAttribute::user_readonly(), frame_pa))
  }
}

trait Index {
  fn l1x(&self) -> usize;
  fn l2x(&self) -> usize;
  fn l3x(&self) -> usize;
}

impl Index for usize {
  fn l1x(&self) -> usize {
    (self >> PAGE_TABLE_L1_SHIFT) & (PAGE_SIZE / MACHINE_SIZE - 1)
  }
  fn l2x(&self) -> usize {
    (self >> PAGE_TABLE_L2_SHIFT) & (PAGE_SIZE / MACHINE_SIZE - 1)
  }
  fn l3x(&self) -> usize {
    (self >> PAGE_TABLE_L3_SHIFT) & (PAGE_SIZE / MACHINE_SIZE - 1)
  }
}

impl core::convert::From<Aarch64PageTableEntry> for Entry {
  fn from(u: Aarch64PageTableEntry) -> Self {
    use tock_registers::*;
    let reg = LocalRegisterCopy::<u64, PAGE_DESCRIPTOR::Register>::new(u.0 as u64);
    Entry::new(EntryAttribute::new(
      reg.matches_all(PAGE_DESCRIPTOR::AP::RW_EL1) || reg.matches_all(PAGE_DESCRIPTOR::AP::RW_EL1_EL0),
      reg.matches_all(PAGE_DESCRIPTOR::AP::RW_EL1_EL0) || reg.matches_all(PAGE_DESCRIPTOR::AP::RO_EL1_EL0),
      reg.matches_all(PAGE_DESCRIPTOR::AttrIndx::DEVICE),
      !reg.is_set(PAGE_DESCRIPTOR::PXN),
      !reg.is_set(PAGE_DESCRIPTOR::UXN),
      reg.is_set(PAGE_DESCRIPTOR::COW),
      reg.is_set(PAGE_DESCRIPTOR::LIB),
    ), (reg.read(PAGE_DESCRIPTOR::OUTPUT_PPN) as usize) << PAGE_SHIFT)
  }
}

impl core::convert::From<Entry> for Aarch64PageTableEntry {
  fn from(pte: Entry) -> Self {
    Aarch64PageTableEntry((
      if pte.attribute().u_shared() { PAGE_DESCRIPTOR::LIB::True } else { PAGE_DESCRIPTOR::LIB::False }
        + if pte.attribute().u_copy_on_write() { PAGE_DESCRIPTOR::COW::True } else { PAGE_DESCRIPTOR::COW::False }
        + if pte.attribute().u_executable() { PAGE_DESCRIPTOR::UXN::False } else { PAGE_DESCRIPTOR::UXN::True }
        + if pte.attribute().k_executable() { PAGE_DESCRIPTOR::PXN::False } else { PAGE_DESCRIPTOR::PXN::True }
        + if pte.attribute().device() {
        PAGE_DESCRIPTOR::SH::OuterShareable + PAGE_DESCRIPTOR::AttrIndx::DEVICE
      } else {
        PAGE_DESCRIPTOR::SH::InnerShareable + PAGE_DESCRIPTOR::AttrIndx::NORMAL
      }
        + if pte.attribute().writable() && pte.attribute().u_readable() {
        PAGE_DESCRIPTOR::AP::RW_EL1_EL0
      } else if pte.attribute().writable() && !pte.attribute().u_readable() {
        PAGE_DESCRIPTOR::AP::RW_EL1
      } else if !pte.attribute().writable() && pte.attribute().u_readable() {
        PAGE_DESCRIPTOR::AP::RO_EL1_EL0
      } else {// if !pte.attr.writable() && !pte.attr.u_readable() {
        PAGE_DESCRIPTOR::AP::RO_EL1
      }
        + PAGE_DESCRIPTOR::TYPE::Table
        + PAGE_DESCRIPTOR::VALID::True
        + PAGE_DESCRIPTOR::OUTPUT_PPN.val((pte.ppn()) as u64)
        + PAGE_DESCRIPTOR::AF::True
    ).value as usize)
  }
}

impl PageTableTrait for Aarch64PageTable {
  fn new(directory: PhysicalFrame) -> Self {
    Aarch64PageTable {
      directory,
      pages: Mutex::new(Vec::new()),
      user_pages: Mutex::new(Vec::new()),
    }
  }

  fn base_pa(&self) -> usize {
    self.directory.pa()
  }

  fn map(&self, va: usize, pa: usize, attr: EntryAttribute) -> Result<(), Error> {
    let directory = Aarch64PageTableEntry::from_pa(self.directory.pa());
    let mut l1e = directory.entry(va.l1x());
    if !l1e.valid() {
      let frame = crate::mm::page_pool::page_alloc()?;
      l1e = Aarch64PageTableEntry::make_table(frame.pa());
      let mut pages = self.pages.lock();
      pages.push(frame);
      directory.set_entry(va.l1x(), l1e);
    }
    let mut l2e = l1e.entry(va.l2x());
    if !l2e.valid() {
      let frame = crate::mm::page_pool::page_alloc()?;
      l2e = Aarch64PageTableEntry::make_table(frame.pa());
      let mut pages = self.pages.lock();
      pages.push(frame);
      l1e.set_entry(va.l2x(), l2e);
    }
    l2e.set_entry(va.l3x(), Aarch64PageTableEntry::from(Entry::new(attr, pa)));
    Ok(())
  }

  fn unmap(&self, va: usize) {
    let directory = Aarch64PageTableEntry::from_pa(self.directory.pa());
    let l1e = directory.entry(va.l1x());
    assert!(l1e.valid());
    let l2e = l1e.entry(va.l2x());
    assert!(l2e.valid());
    l2e.set_entry(va.l3x(), Aarch64PageTableEntry(0));
  }

  fn insert_page(&self, va: usize, user_frame: crate::mm::Frame, attr: EntryAttribute) -> Result<(), Error> {
    let pa = user_frame.pa();
    let mut user_frames = self.user_pages.lock();
    user_frames.push(user_frame);
    if let Some(p) = self.lookup_page(va) {
      if p.pa() != pa {
        // replace mapped frame
        self.remove_page(va)?;
      }
    }
    crate::arch::Arch::invalidate_tlb();
    self.map(va, pa, attr)?;
    Ok(())
  }

  fn lookup_page(&self, va: usize) -> Option<Entry> {
    let directory = Aarch64PageTableEntry::from_pa(self.directory.pa());
    let l1e = directory.entry(va.l1x());
    if !l1e.valid() {
      return None;
    }
    let l2e = l1e.entry(va.l2x());
    if !l2e.valid() {
      return None;
    }
    let l3e = l2e.entry(va.l3x());
    if l3e.valid() {
      Some(Entry::from(l3e))
    } else {
      None
    }
  }

  fn lookup_user_page(&self, va: usize) -> Option<Frame> {
    match self.lookup_page(va) {
      None => { None }
      Some(entry) => {
        let pa = entry.pa();
        let user_frames = self.user_pages.lock();
        for uf in user_frames.iter() {
          if uf.pa() == pa {
            return Some(uf.clone());
          }
        }
        None
      }
    }
  }

  fn remove_page(&self, va: usize) -> Result<(), Error> {
    if let Some(_) = self.lookup_page(va) {
      self.unmap(va);
      crate::arch::Arch::invalidate_tlb();
      Ok(())
    } else {
      Err(ERROR_INVARG)
    }
  }

  fn recursive_map(&self, va: usize) {
    assert_eq!(va % (1 << PAGE_TABLE_L1_SHIFT), 0);
    let directory = Aarch64PageTableEntry::from_pa(self.directory.pa());
    let l1x = va / (1 << PAGE_TABLE_L1_SHIFT);
    directory.set_entry(l1x, Aarch64PageTableEntry::from(Entry::new(
      EntryAttribute::user_readonly(),
      self.directory.pa(),
    )));
  }

  fn install_user_page_table(base: usize, asid: AddressSpaceId) {
    use cortex_a::registers::TTBR0_EL1;
    use cortex_a::asm::barrier::*;
    unsafe {
      TTBR0_EL1.write(TTBR0_EL1::ASID.val(asid as u64));
      TTBR0_EL1.set_baddr(base as u64);
      isb(SY);
      dsb(SY);
    }
  }
}
