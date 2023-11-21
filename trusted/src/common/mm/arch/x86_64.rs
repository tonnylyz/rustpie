use x86_64::structures::paging::{PageTable as PT, PageTableFlags};
// use x86_64::structures::paging::{RecursivePageTable, Translate};
// use x86_64::VirtAddr;

const PAGE_SHIFT: usize = 12;
const LEVEL_SHIFT: usize = 9;
const LEVEL_ENTRY: usize = 1 << LEVEL_SHIFT;

pub fn va_to_pa(va: usize) -> Option<usize> {
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
  if l4pt[l4i].flags().contains(PageTableFlags::PRESENT) {
    let mut l3pt = l4i << PAGE_SHIFT;
    l3pt |= recur_index << (PAGE_SHIFT + 1 * LEVEL_SHIFT);
    l3pt |= recur_index << (PAGE_SHIFT + 2 * LEVEL_SHIFT);
    l3pt |= recur_index << (PAGE_SHIFT + 3 * LEVEL_SHIFT);
    trace!("l4 idx {} l3pt {:X}", l4i, l3pt);
    let l3pt = unsafe { (l3pt as *mut PT).as_mut().unwrap() };
    let l3i = (va >> (PAGE_SHIFT + 2 * LEVEL_SHIFT)) & (LEVEL_ENTRY - 1);
    if l3pt[l3i].flags().contains(PageTableFlags::PRESENT) {
      let mut l2pt = l3i << PAGE_SHIFT;
      l2pt |= l4i << (PAGE_SHIFT + 1 * LEVEL_SHIFT);
      l2pt |= recur_index << (PAGE_SHIFT + 2 * LEVEL_SHIFT);
      l2pt |= recur_index << (PAGE_SHIFT + 3 * LEVEL_SHIFT);
      trace!("l3 idx {} l2pt {:X}", l3i, l2pt);
      let l2pt = unsafe { (l2pt as *mut PT).as_mut().unwrap() };
      let l2i = (va >> (PAGE_SHIFT + 1 * LEVEL_SHIFT)) & (LEVEL_ENTRY - 1);
      if l2pt[l2i].flags().contains(PageTableFlags::PRESENT) {
        let mut l1pt = l2i << PAGE_SHIFT;
        l1pt |= l3i << (PAGE_SHIFT + 1 * LEVEL_SHIFT);
        l1pt |= l4i << (PAGE_SHIFT + 2 * LEVEL_SHIFT);
        l1pt |= recur_index << (PAGE_SHIFT + 3 * LEVEL_SHIFT);
        trace!("l2 idx {} l1pt {:X}", l2i, l1pt);
        let l1pt = unsafe { (l1pt as *mut PT).as_mut().unwrap() };
        let l1i = (va >> (PAGE_SHIFT)) & (LEVEL_ENTRY - 1);
        trace!("l1 idx {} l1e {:?}", l1i, l1pt[l1i]);
        if l1pt[l1i].flags().contains(PageTableFlags::PRESENT) {
          return Some(l1pt[l1i].addr().as_u64() as usize);
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

