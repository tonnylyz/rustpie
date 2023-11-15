use crate::arch::*;
use crate::kernel::traits::*;
use crate::mm::page_table::{
  Entry, EntryAttribute, Error, PageTableEntryAttrTrait, PageTableTrait,
};
use crate::mm::{Frame, PhysicalFrame};

#[derive(Debug)]
pub struct X64PageTable {
  directory: PhysicalFrame,
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
pub struct X64PageTableEntry(usize);

impl ArchPageTableEntryTrait for X64PageTableEntry {
  fn from_pte(value: usize) -> Self {
    todo!()
  }

  fn from_pa(pa: usize) -> Self {
    todo!()
  }

  fn to_pte(&self) -> usize {
    todo!()
  }

  fn to_pa(&self) -> usize {
    todo!()
  }

  fn to_kva(&self) -> usize {
    todo!()
  }

  fn valid(&self) -> bool {
    todo!()
  }

  fn entry(&self, index: usize) -> Self {
    todo!()
  }

  fn set_entry(&self, index: usize, value: Self) {
    todo!()
  }

  fn make_table(frame_pa: usize) -> Self {
    todo!()
  }
}

impl core::convert::From<X64PageTableEntry> for Entry {
  fn from(value: X64PageTableEntry) -> Self {
    todo!()
  }
}

impl core::convert::From<Entry> for X64PageTableEntry {
  fn from(value: Entry) -> Self {
    todo!()
  }
}

impl PageTableTrait for X64PageTable {
  fn new(directory: crate::mm::PhysicalFrame) -> Self {
    todo!()
  }

  fn base_pa(&self) -> usize {
    todo!()
  }

  fn map(&self, va: usize, pa: usize, attr: EntryAttribute) -> Result<(), Error> {
    todo!()
  }

  fn unmap(&self, va: usize) {
    todo!()
  }

  fn insert_page(
    &self,
    va: usize,
    user_frame: crate::mm::Frame,
    attr: EntryAttribute,
  ) -> Result<(), Error> {
    todo!()
  }

  fn lookup_page(&self, va: usize) -> Option<Entry> {
    todo!()
  }

  fn lookup_user_page(&self, va: usize) -> Option<Frame> {
    todo!()
  }

  fn remove_page(&self, va: usize) -> Result<(), Error> {
    todo!()
  }

  fn recursive_map(&self, va: usize) {
    todo!()
  }

  fn install_user_page_table(base: usize, asid: AddressSpaceId) {
    todo!()
  }
}
