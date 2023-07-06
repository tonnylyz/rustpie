use rpabi::{PAGE_SIZE, PAGE_TABLE_L1_SHIFT, PAGE_TABLE_L2_SHIFT, PAGE_TABLE_L3_SHIFT};
use rpabi::CONFIG_READ_ONLY_LEVEL_1_PAGE_TABLE_BTM;
use rpabi::CONFIG_READ_ONLY_LEVEL_2_PAGE_TABLE_BTM;
use rpabi::CONFIG_READ_ONLY_LEVEL_3_PAGE_TABLE_BTM;
use rpabi::mm::vm_descriptor::*;
use tock_registers::LocalRegisterCopy;

use crate::libtrusted::mm::PageAttribute;

fn read_directory_entry(l1_index: usize) -> usize {
  let ppte = CONFIG_READ_ONLY_LEVEL_1_PAGE_TABLE_BTM + l1_index * 8;
  unsafe { (ppte as *const usize).read_volatile() }
}

fn read_level_1_entry(l1_index: usize, l2_index: usize) -> usize {
  let ppte = CONFIG_READ_ONLY_LEVEL_2_PAGE_TABLE_BTM + l1_index * PAGE_SIZE + l2_index * 8;
  unsafe { (ppte as *const usize).read_volatile() }
}

fn read_level_2_entry(l1_index: usize, l2_index: usize, l3_index: usize) -> usize {
  let ppte = CONFIG_READ_ONLY_LEVEL_3_PAGE_TABLE_BTM + l1_index * PAGE_SIZE * 512 + l2_index * PAGE_SIZE + l3_index * 8;
  unsafe { (ppte as *const usize).read_volatile() }
}

#[derive(Clone, Copy, Debug)]
pub struct Entry(usize);

impl Default for Entry {
  fn default() -> Self {
    Entry::new(true, true, false, false)
  }
}

impl Entry {
  pub fn new(writable: bool, executable: bool, copy_on_write: bool, shared: bool) -> Self {
    Entry((PAGE_DESCRIPTOR::USER::True
      + PAGE_DESCRIPTOR::R::True
      + if writable {
      PAGE_DESCRIPTOR::W::True
    } else {
      PAGE_DESCRIPTOR::W::False
    } + if executable {
      PAGE_DESCRIPTOR::X::True
    } else {
      PAGE_DESCRIPTOR::X::False
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

const PTE_ATTR_MASK: usize = 0x3ff;

impl PageAttribute for Entry {
  fn executable(&self) -> bool {
    self.reg().is_set(PAGE_DESCRIPTOR::X)
  }

  fn writable(&self) -> bool {
    self.reg().is_set(PAGE_DESCRIPTOR::W)
  }

  fn copy_on_write(&self) -> bool {
    self.reg().is_set(PAGE_DESCRIPTOR::COW)
  }

  fn shared(&self) -> bool {
    self.reg().is_set(PAGE_DESCRIPTOR::LIB)
  }

  fn set_executable(&mut self, b: bool) {
    if b {
      self.0 |= (PAGE_DESCRIPTOR::X::True).value as usize;
    } else {
      self.0 &= !(PAGE_DESCRIPTOR::X::True).value as usize;
    }
  }

  fn set_writable(&mut self, b: bool) {
    if b {
      self.0 |= (PAGE_DESCRIPTOR::W::True).value as usize;
    } else {
      self.0 &= !(PAGE_DESCRIPTOR::W::True).value as usize;
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
    (self.0 >> 10) << 12
  }

  fn set_address(&mut self, addr: usize) {
    let ppn = addr >> 12;
    self.0 &= PTE_ATTR_MASK;
    self.0 |= ppn << 10;
  }

  fn attribute(&self) -> usize {
    self.0 | PTE_ATTR_MASK
  }

  fn set_attribute(&mut self, attr: usize) {
    self.0 &= !PTE_ATTR_MASK;
    self.0 |= attr & PTE_ATTR_MASK;
  }

  fn is_valid(&self) -> bool {
    self.reg().is_set(PAGE_DESCRIPTOR::VALID)
  }

  fn is_table(&self) -> bool {
    self.is_valid() && self.reg().matches_all(PAGE_DESCRIPTOR::R::False)
      && self.reg().matches_all(PAGE_DESCRIPTOR::W::False)
      && self.reg().matches_all(PAGE_DESCRIPTOR::X::False)
  }

  fn is_page(&self) -> bool {
    self.is_valid() && self.reg().is_set(PAGE_DESCRIPTOR::USER) && !self.is_table()
  }
}

pub fn query(va: usize) -> Option<Entry> {
  let l1x = (va >> PAGE_TABLE_L1_SHIFT) & (PAGE_SIZE / core::mem::size_of::<usize>() - 1);
  let l2x = (va >> PAGE_TABLE_L2_SHIFT) & (PAGE_SIZE / core::mem::size_of::<usize>() - 1);
  let l3x = (va >> PAGE_TABLE_L3_SHIFT) & (PAGE_SIZE / core::mem::size_of::<usize>() - 1);
  if read_directory_entry(l1x) & 0b1 != 0 {
    if read_level_1_entry(l1x, l2x) & 0b1 != 0 {
      let r = read_level_2_entry(l1x, l2x, l3x);
      if r & 0b1 != 0 {
        Some(Entry(r))
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

pub fn traverse<F>(limit: usize, f: F) where F: Fn(usize, Entry) -> () {
  for l1x in 0..(PAGE_SIZE / core::mem::size_of::<usize>()) {
    let l1e = read_directory_entry(l1x);
    if l1e & 0b1 == 0 || l1e & (1 << 4) == 0 {
      continue;
    }
    for l2x in 0..(PAGE_SIZE / core::mem::size_of::<usize>()) {
      let va = (l1x << PAGE_TABLE_L1_SHIFT) + (l2x << PAGE_TABLE_L2_SHIFT);
      if va >= limit {
        return;
      }
      let l2e = read_level_1_entry(l1x, l2x);
      if l2e & 0b1 == 0 {
        continue;
      }
      for l3x in 0..(PAGE_SIZE / core::mem::size_of::<usize>()) {
        let va = (l1x << PAGE_TABLE_L1_SHIFT) + (l2x << PAGE_TABLE_L2_SHIFT) + (l3x << PAGE_TABLE_L3_SHIFT);
        if va >= limit {
          return;
        }
        let l3e = read_level_2_entry(l1x, l2x, l3x);
        if l3e & 0b1 != 0 {
          f(va, Entry(l3e));
        }
      }
    }
  }
}