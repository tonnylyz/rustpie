use crate::config::*;
use crate::traits::EntryLike;

use super::vm_descriptor::*;
use register::*;

const RECURSIVE_PAGE_TABLE_BTM: usize = 0x3f_c000_0000;
const PTE_ADDR_MASK: usize = 0x0000_FFFF_FFFF_F000;
const PTE_ATTR_MASK: usize = !PTE_ADDR_MASK;


fn read_directory_entry(l1_index: usize) -> usize {
  let l1x = RECURSIVE_PAGE_TABLE_BTM >> PAGE_TABLE_L1_SHIFT;
  let l2x = RECURSIVE_PAGE_TABLE_BTM >> PAGE_TABLE_L1_SHIFT;
  let l3x = l1_index;
  let ppte = RECURSIVE_PAGE_TABLE_BTM + l1x * (1 << PAGE_TABLE_L2_SHIFT) + l2x * (1 << PAGE_TABLE_L3_SHIFT) + l3x * (1 << WORD_SHIFT);
  unsafe { (ppte as *const usize).read_volatile() }
}

fn read_level_1_entry(l1_index: usize, l2_index: usize) -> usize {
  let l1x = RECURSIVE_PAGE_TABLE_BTM >> PAGE_TABLE_L1_SHIFT;
  let l2x = l1_index;
  let l3x = l2_index;
  let ppte = RECURSIVE_PAGE_TABLE_BTM + l1x * (1 << PAGE_TABLE_L2_SHIFT) + l2x * (1 << PAGE_TABLE_L3_SHIFT) + l3x * (1 << WORD_SHIFT);
  unsafe { (ppte as *const usize).read_volatile() }
}

fn read_level_2_entry(l1_index: usize, l2_index: usize, l3_index: usize) -> usize {
  let l1x = l1_index;
  let l2x = l2_index;
  let l3x = l3_index;
  let ppte = RECURSIVE_PAGE_TABLE_BTM + l1x * (1 << PAGE_TABLE_L2_SHIFT) + l2x * (1 << PAGE_TABLE_L3_SHIFT) + l3x * (1 << WORD_SHIFT);
  unsafe { (ppte as *const usize).read_volatile() }
}

fn read_page_table_entry(va: usize) -> Option<usize> {
  let l1x = (va >> PAGE_TABLE_L1_SHIFT) & (PAGE_SIZE / WORD_SIZE - 1);
  let l2x = (va >> PAGE_TABLE_L2_SHIFT) & (PAGE_SIZE / WORD_SIZE - 1);
  let l3x = (va >> PAGE_TABLE_L3_SHIFT) & (PAGE_SIZE / WORD_SIZE - 1);
  if read_directory_entry(l1x) & 0b11 != 0 {
    if read_level_1_entry(l1x, l2x) & 0b11 != 0 {
      let r = read_level_2_entry(l1x, l2x, l3x);
      if r & 0b11 != 0 {
        Some(r)
      } else {
        None
      }
    } else {
      None
    }
  } else {
    None
  }
}

#[derive(Clone, Copy, Debug)]
pub struct Entry(usize);

impl Default for Entry {
  fn default() -> Self {
    Entry::attr(true, true, false, false)
  }
}

impl Entry {
  pub fn attr(writable: bool, executable: bool, copy_on_write: bool, shared: bool) -> Self {
    Entry((if writable {
      PAGE_DESCRIPTOR::AP::RW_EL1_EL0
    } else {
      PAGE_DESCRIPTOR::AP::RO_EL1_EL0
    } + if executable {
      PAGE_DESCRIPTOR::UXN::False
    } else {
      PAGE_DESCRIPTOR::UXN::True
    } + if copy_on_write {
      PAGE_DESCRIPTOR::COW::True
    } else {
      PAGE_DESCRIPTOR::COW::False
    } + if shared {
      PAGE_DESCRIPTOR::LIB::True
    } else {
      PAGE_DESCRIPTOR::LIB::False
    }).value as usize)
  }

  pub fn reg(&self) -> LocalRegisterCopy<u64, PAGE_DESCRIPTOR::Register> {
    LocalRegisterCopy::new(self.0 as u64)
  }
}

impl EntryLike for Entry {
  fn executable(&self) -> bool {
    self.reg().is_set(PAGE_DESCRIPTOR::UXN)
  }

  fn writable(&self) -> bool {
    self.reg().matches_all(PAGE_DESCRIPTOR::AP::RW_EL1_EL0)
  }

  fn copy_on_write(&self) -> bool {
    self.reg().is_set(PAGE_DESCRIPTOR::COW)
  }

  fn shared(&self) -> bool {
    self.reg().is_set(PAGE_DESCRIPTOR::LIB)
  }

  fn set_executable(&mut self, b: bool) {
    if b {
      self.0 &= !(PAGE_DESCRIPTOR::UXN::True).value as usize;
    } else {
      self.0 |= (PAGE_DESCRIPTOR::UXN::True).value as usize;
    }
  }

  fn set_writable(&mut self, b: bool) {
    if b {
      self.0 &= !(PAGE_DESCRIPTOR::AP.val(0b11)).value as usize;
      self.0 |= (PAGE_DESCRIPTOR::AP::RW_EL1_EL0).value as usize;
    } else {
      self.0 |= (PAGE_DESCRIPTOR::AP::RO_EL1_EL0).value  as usize;
    }
  }

  fn set_copy_on_write(&mut self, b: bool) {
    if b {
      self.0 |= (PAGE_DESCRIPTOR::COW::True).value as usize;
    } else {
      self.0 &= !(PAGE_DESCRIPTOR::COW::True).value as usize;
    }
  }

  fn set_shared(&mut self, b: bool) {
    if b {
      self.0 |= (PAGE_DESCRIPTOR::LIB::True).value as usize;
    } else {
      self.0 &= !(PAGE_DESCRIPTOR::LIB::True).value as usize;
    }
  }

  fn address(&self) -> usize {
    (self.0 & PTE_ADDR_MASK) as usize
  }

  fn set_address(&mut self, addr: usize) {
    self.0 = (self.0 & !PTE_ADDR_MASK) | (addr & PTE_ADDR_MASK)
  }

  fn attribute(&self) -> usize {
    (self.0 & PTE_ATTR_MASK) as usize
  }

  fn set_attribute(&mut self, attr: usize) {
    self.0 = (self.0 & !PTE_ATTR_MASK) | (attr & PTE_ATTR_MASK)
  }

  fn is_valid(&self) -> bool {
    self.reg().is_set(PAGE_DESCRIPTOR::VALID)
  }

  fn is_table(&self) -> bool {
    // NOTE: is_table depends on translation level
    self.is_valid() && self.reg().matches_all(PAGE_DESCRIPTOR::TYPE::Table)
  }

  fn is_page(&self) -> bool {
    // NOTE: only 4KB page
    self.is_valid() && self.reg().matches_all(PAGE_DESCRIPTOR::TYPE::Table)
  }
}

pub fn query(va: usize) -> Option<Entry> {
  if let Some(pte) = read_page_table_entry(va) {
    Some(Entry(pte))
  } else {
    None
  }
}

pub fn traverse<F>(limit: usize, f: F) where F: Fn(usize, Entry) -> () {
  for l1x in 0..(PAGE_SIZE / WORD_SIZE) {
    let l1e = read_directory_entry(l1x);
    if l1e & 0b11 == 0 {
      continue;
    }
    for l2x in 0..(PAGE_SIZE / WORD_SIZE) {
      let l2e = read_level_1_entry(l1x, l2x);
      if l2e & 0b11 == 0 {
        continue;
      }
      for l3x in 0..(PAGE_SIZE / WORD_SIZE) {
        let va = (l1x << PAGE_TABLE_L1_SHIFT) + (l2x << PAGE_TABLE_L2_SHIFT) + (l3x << PAGE_TABLE_L3_SHIFT);
        if va >= limit {
          return;
        }
        let l3e = read_level_2_entry(l1x, l2x, l3x);
        if l3e & 0b11 != 0 {
          f(va, Entry(l3e));
        }
      }
    }
  }
}