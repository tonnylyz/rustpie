use rpabi::{CONFIG_RECURSIVE_PAGE_TABLE_BTM, PAGE_SIZE, WORD_SHIFT};

const PTE_ADDR_MASK: usize = 0x0000_FFFF_FFFF_F000;

pub const PAGE_TABLE_L1_SHIFT: usize = 30;
pub const PAGE_TABLE_L2_SHIFT: usize = 21;
pub const PAGE_TABLE_L3_SHIFT: usize = 12;
const RECURSIVE_INDEX: usize = CONFIG_RECURSIVE_PAGE_TABLE_BTM >> 30;

fn recursive_pt(a: usize, b: usize, c: usize) -> usize {
  let ppte = CONFIG_RECURSIVE_PAGE_TABLE_BTM + a * (1 << PAGE_TABLE_L2_SHIFT) + b * (1 << PAGE_TABLE_L3_SHIFT) + c * (1 << WORD_SHIFT);
  unsafe { (ppte as *const usize).read_volatile() }
}

pub fn va_to_pa(va: usize) -> Option<usize> {
  // 39-bit VA [ 0 ][ a ][ b ][ c ][ page offset ]
  let a = (va >> PAGE_TABLE_L1_SHIFT) & (PAGE_SIZE / core::mem::size_of::<usize>() - 1);
  let b = (va >> PAGE_TABLE_L2_SHIFT) & (PAGE_SIZE / core::mem::size_of::<usize>() - 1);
  let c = (va >> PAGE_TABLE_L3_SHIFT) & (PAGE_SIZE / core::mem::size_of::<usize>() - 1);
  if recursive_pt(RECURSIVE_INDEX, RECURSIVE_INDEX, a) & 0b11 == 0 {
    return None;
  }
  if recursive_pt(RECURSIVE_INDEX, a, b) & 0b11 == 0 {
    return None;
  }
  let pte = recursive_pt(a, b, c);
  if pte & 0b11 == 0 {
    return None;
  }
  Some(pte & PTE_ADDR_MASK)
}
