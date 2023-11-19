use super::PageAttribute;
use x86_64::structures::paging::{PageTable as PT, PageTableFlags};
// use x86_64::structures::paging::{RecursivePageTable, Translate};
// use x86_64::VirtAddr;

const PTE_ADDR_MASK: usize = 0x0000_FFFF_FFFF_F000;
const PAGE_SHIFT: usize = 12;
const LEVEL_SHIFT: usize = 9;
const LEVEL_ENTRY: usize = 1 << LEVEL_SHIFT;

#[derive(Clone, Copy, Debug)]
pub struct Entry(usize);

impl Entry {
  pub fn new(writable: bool, _executable: bool, copy_on_write: bool, shared: bool) -> Self {
    let mut flags = PageTableFlags::PRESENT;
    if shared {
      flags |= PageTableFlags::BIT_10;
    }
    if copy_on_write {
      flags |= PageTableFlags::BIT_9;
    }
    if writable {
      flags |= PageTableFlags::WRITABLE;
    }
    Entry(flags.bits() as usize)
  }
}

impl PageAttribute for Entry {
  fn executable(&self) -> bool {
    true
  }

  fn writable(&self) -> bool {
    let flags = PageTableFlags::from_bits(self.0 as u64).unwrap();
    flags.contains(PageTableFlags::WRITABLE)
  }

  fn copy_on_write(&self) -> bool {
    let flags = PageTableFlags::from_bits(self.0 as u64).unwrap();
    flags.contains(PageTableFlags::BIT_9)
  }

  fn shared(&self) -> bool {
    let flags = PageTableFlags::from_bits(self.0 as u64).unwrap();
    flags.contains(PageTableFlags::BIT_10)
  }

  fn set_executable(&mut self, _b: bool) {}

  fn set_writable(&mut self, b: bool) {
    let mut flags = PageTableFlags::from_bits(self.0 as u64).unwrap();
    flags |= PageTableFlags::WRITABLE;
    self.0 = flags.bits() as usize;
  }

  fn set_copy_on_write(&mut self, b: bool) {
    let mut flags = PageTableFlags::from_bits(self.0 as u64).unwrap();
    flags |= PageTableFlags::BIT_9;
    self.0 = flags.bits() as usize;
  }

  fn set_shared(&mut self, b: bool) {
    let mut flags = PageTableFlags::from_bits(self.0 as u64).unwrap();
    flags |= PageTableFlags::BIT_10;
    self.0 = flags.bits() as usize;
  }

  fn address(&self) -> usize {
    self.0 & PTE_ADDR_MASK
  }

  fn set_address(&mut self, addr: usize) {
    self.0 &= !PTE_ADDR_MASK;
    self.0 |= addr & PTE_ADDR_MASK;
  }

  fn attribute(&self) -> usize {
    self.0 & !PTE_ADDR_MASK
  }

  fn set_attribute(&mut self, attr: usize) {
    self.0 &= PTE_ADDR_MASK;
    self.0 |= attr;
  }

  fn is_valid(&self) -> bool {
    let flags = PageTableFlags::from_bits(self.0 as u64).unwrap();
    flags.contains(PageTableFlags::PRESENT)
  }

  fn is_table(&self) -> bool {
    let flags = PageTableFlags::from_bits(self.0 as u64).unwrap();
    !flags.contains(PageTableFlags::HUGE_PAGE)
  }

  fn is_page(&self) -> bool {
    !self.is_table()
  }
}

pub fn query(va: usize) -> Option<Entry> {
  trace!("va {:x}", va);
  // use x86_64::structures::paging::mapper::TranslateResult::*;
  let recur_index = rpabi::CONFIG_RECURSIVE_PAGE_TABLE_BTM >> (PAGE_SHIFT + 3 * LEVEL_SHIFT);
  let mut l4pt = recur_index << PAGE_SHIFT;
  l4pt |= recur_index << (PAGE_SHIFT + 1 * LEVEL_SHIFT);
  l4pt |= recur_index << (PAGE_SHIFT + 2 * LEVEL_SHIFT);
  l4pt |= recur_index << (PAGE_SHIFT + 3 * LEVEL_SHIFT);
  trace!("self map idx {} l4pt {:X}", recur_index, l4pt);
  let l4pt = unsafe { (l4pt as *mut PT).as_mut().unwrap() };
  let l4i = (va >> (PAGE_SHIFT + 3 * LEVEL_SHIFT)) & (LEVEL_ENTRY - 1);
  if Entry(l4pt[l4i].flags().bits() as usize).is_valid() {
    let mut l3pt = l4i << PAGE_SHIFT;
    l3pt |= recur_index << (PAGE_SHIFT + 1 * LEVEL_SHIFT);
    l3pt |= recur_index << (PAGE_SHIFT + 2 * LEVEL_SHIFT);
    l3pt |= recur_index << (PAGE_SHIFT + 3 * LEVEL_SHIFT);
    trace!("l4 idx {} l3pt {:X}", l4i, l3pt);
    let l3pt = unsafe { (l3pt as *mut PT).as_mut().unwrap() };
    let l3i = (va >> (PAGE_SHIFT + 2 * LEVEL_SHIFT)) & (LEVEL_ENTRY - 1);
    if Entry(l3pt[l3i].flags().bits() as usize).is_valid() {
      let mut l2pt = l3i << PAGE_SHIFT;
      l2pt |= l4i << (PAGE_SHIFT + 1 * LEVEL_SHIFT);
      l2pt |= recur_index << (PAGE_SHIFT + 2 * LEVEL_SHIFT);
      l2pt |= recur_index << (PAGE_SHIFT + 3 * LEVEL_SHIFT);
      trace!("l3 idx {} l2pt {:X}", l3i, l2pt);
      let l2pt = unsafe { (l2pt as *mut PT).as_mut().unwrap() };
      let l2i = (va >> (PAGE_SHIFT + 1 * LEVEL_SHIFT)) & (LEVEL_ENTRY - 1);
      if Entry(l2pt[l2i].flags().bits() as usize).is_valid() {
        let mut l1pt = l2i << PAGE_SHIFT;
        l1pt |= l3i << (PAGE_SHIFT + 1 * LEVEL_SHIFT);
        l1pt |= l4i << (PAGE_SHIFT + 2 * LEVEL_SHIFT);
        l1pt |= recur_index << (PAGE_SHIFT + 3 * LEVEL_SHIFT);
        trace!("l2 idx {} l1pt {:X}", l2i, l1pt);
        let l1pt = unsafe { (l1pt as *mut PT).as_mut().unwrap() };
        let l1i = (va >> (PAGE_SHIFT)) & (LEVEL_ENTRY - 1);
        trace!("l1 idx {} l1e {:?}", l1i, l1pt[l1i]);
        if Entry(l1pt[l1i].flags().bits() as usize).is_valid() {
          return Some(Entry(l1pt[l1i].addr().as_u64() as usize | l1pt[l1i].flags().bits() as usize));
        }
      }
    }
  }
  return None;
  // let recursive_pt = RecursivePageTable::new(l4pt).unwrap();
  // match recursive_pt.translate(VirtAddr::new(va as u64)) {
  //   Mapped {
  //     frame,
  //     offset,
  //     flags,
  //   } => Some(Entry(
  //     frame.start_address().as_u64() as usize | flags.bits() as usize,
  //   )),
  //   _ => None,
  // }
}

#[allow(dead_code)]
pub fn traverse<F>(limit: usize, f: F)
where
  F: Fn(usize, Entry) -> (),
{
  unimplemented!()
}
