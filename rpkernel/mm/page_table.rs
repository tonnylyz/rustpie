use core::fmt::{Display, Formatter};

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::arch::PAGE_SHIFT;
use crate::mm::{Frame, PhysicalFrame};

use rpabi::syscall::mm::*;

pub trait ArchPageTableEntryTrait {
  fn from_pte(value: usize) -> Self;
  fn from_pa(pa: usize) -> Self;
  fn to_pte(&self) -> usize;
  fn to_pa(&self) -> usize;
  fn to_kva(&self) -> usize;
  fn valid(&self) -> bool;
  fn entry(&self, index: usize) -> Self;
  fn set_entry(&self, index: usize, value: Self);
  fn make_table(frame_pa: usize) -> Self;
}

#[derive(Copy, Clone, Debug)]
pub struct Entry {
  attribute: EntryAttribute,
  pa: usize,
}

impl Entry {
  pub fn new(attribute: EntryAttribute, pa: usize) -> Self {
    Entry { attribute, pa }
  }
  pub fn attribute(&self) -> EntryAttribute {
    self.attribute
  }
  pub fn pa(&self) -> usize {
    self.pa
  }
  pub fn ppn(&self) -> usize {
    self.pa >> PAGE_SHIFT
  }
}

impl Display for Entry {
  fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
    write!(f, "PTE [{:016x}] {}", self.pa, self.attribute)
  }
}

pub type Error = usize;

pub trait PageTableTrait {
  fn new(directory_kva: usize, table_frames: &mut Vec<PhysicalFrame>) -> Self;
  fn map(
    &self,
    va: usize,
    pa: usize,
    attr: EntryAttribute,
    table_frames: &mut Vec<PhysicalFrame>,
  ) -> Result<(), Error>;
  fn unmap(&self, va: usize);
  fn lookup_page(&self, va: usize) -> Option<Entry>;
  fn recursive_map(&self, va: usize);
}

#[derive(Debug)]
pub struct GenericPageTable<T>
where
  T: PageTableTrait,
{
  arch_pt: T,
  directory: PhysicalFrame,
  table_frames: Vec<PhysicalFrame>,
  user_frames: BTreeMap<usize, Frame>,
}

impl<T> GenericPageTable<T>
where
  T: PageTableTrait,
{
  pub fn new() -> Result<Self, usize> {
    let directory = super::page_pool::page_alloc()?;
    let mut table_frames = Vec::new();
    let arch_pt = T::new(directory.kva(), &mut table_frames);
    let r = GenericPageTable {
      arch_pt,
      directory,
      table_frames,
      user_frames: BTreeMap::new(),
    };
    Ok(r)
  }

  pub fn insert_page(
    &mut self,
    va: usize,
    user_frame: Frame,
    attr: EntryAttribute,
  ) -> Result<(), Error> {
    let pa = user_frame.pa();
    if let Some(p) = self.arch_pt.lookup_page(va) {
      if p.pa() != pa {
        // replace mapped frame
        self.remove_page(va)?;
      }
    }
    self.arch_pt.map(va, pa, attr, &mut self.table_frames)?;
    self.user_frames.insert(va, user_frame);
    Ok(())
  }

  pub fn lookup_user_page(&self, va: usize) -> Option<Frame> {
    self.user_frames.get(&va).map(|x| x.clone())
  }

  pub fn remove_page(&mut self, va: usize) -> Result<(), Error> {
    if let Some(_) = self.arch_pt.lookup_page(va) {
      self.arch_pt.unmap(va);
      self.user_frames.remove(&va);
      Ok(())
    } else {
      Err(rpabi::syscall::error::ERROR_INVARG)
    }
  }

  pub fn recursive_map(&self, va: usize) {
    self.arch_pt.recursive_map(va);
  }

  pub fn directory_pa(&self) -> usize {
    self.directory.pa()
  }

  pub fn lookup_page(&self, va: usize) -> Option<Entry> {
    self.arch_pt.lookup_page(va)
  }

  pub fn map(&mut self, va: usize, pa: usize, attr: EntryAttribute) -> Result<(), Error> {
    self.arch_pt.map(va, pa, attr, &mut self.table_frames)
  }
}

pub type PageTable = GenericPageTable<crate::arch::PageTable>;
