use rpabi::PAGE_SIZE;
use rpabi::CONFIG_READ_ONLY_LEVEL_1_PAGE_TABLE_BTM;
use rpabi::CONFIG_READ_ONLY_LEVEL_2_PAGE_TABLE_BTM;
use rpabi::CONFIG_READ_ONLY_LEVEL_3_PAGE_TABLE_BTM;

pub const PAGE_TABLE_L1_SHIFT: usize = 30;
pub const PAGE_TABLE_L2_SHIFT: usize = 21;
pub const PAGE_TABLE_L3_SHIFT: usize = 12;

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

const PTE_ATTR_MASK: usize = 0x3ff;
fn entry_to_address(entry: usize) -> usize {
  ((entry & !PTE_ATTR_MASK) >> 10) << 12
}

pub fn va_to_pa(va: usize) -> Option<usize> {
  let l1x = (va >> PAGE_TABLE_L1_SHIFT) & (PAGE_SIZE / core::mem::size_of::<usize>() - 1);
  let l2x = (va >> PAGE_TABLE_L2_SHIFT) & (PAGE_SIZE / core::mem::size_of::<usize>() - 1);
  let l3x = (va >> PAGE_TABLE_L3_SHIFT) & (PAGE_SIZE / core::mem::size_of::<usize>() - 1);
  if read_directory_entry(l1x) & 0b1 == 0 {
    return None;
  }
  if read_level_1_entry(l1x, l2x) & 0b1 == 0 {
    return None;
  }
  let r = read_level_2_entry(l1x, l2x, l3x);
  if r & 0b1 != 0 {
    Some(entry_to_address(r))
  } else {
    None
  }
}
