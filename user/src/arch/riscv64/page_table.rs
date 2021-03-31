use crate::config::*;

use super::vm_descriptor::*;

const CONFIG_READ_ONLY_LEVEL_3_PAGE_TABLE_TOP: usize = 0x40_0000_0000;
// 1 GB
const CONFIG_READ_ONLY_LEVEL_3_PAGE_TABLE_BTM: usize = 0x3f_c000_0000;
// 1 GB
const CONFIG_READ_ONLY_LEVEL_2_PAGE_TABLE_BTM: usize = 0x3f_c000_0000 - 0x20_0000;
// 2 MB
const CONFIG_READ_ONLY_LEVEL_1_PAGE_TABLE_BTM: usize = 0x3f_c000_0000 - 0x20_0000 - 0x1000; // 4 KB

fn read_directory_entry(l1_index: usize) -> u64 {
  let ppte = CONFIG_READ_ONLY_LEVEL_1_PAGE_TABLE_BTM + l1_index * 8;
  unsafe { core::intrinsics::volatile_load(ppte as *const u64) }
}

fn read_level_1_entry(l1_index: usize, l2_index: usize) -> u64 {
  let ppte = CONFIG_READ_ONLY_LEVEL_2_PAGE_TABLE_BTM + l1_index * PAGE_SIZE + l2_index * 8;
  unsafe { core::intrinsics::volatile_load(ppte as *const u64) }
}

fn read_level_2_entry(l1_index: usize, l2_index: usize, l3_index: usize) -> u64 {
  let ppte = CONFIG_READ_ONLY_LEVEL_3_PAGE_TABLE_BTM + l1_index * PAGE_SIZE * 512 + l2_index * PAGE_SIZE + l3_index * 8;
  unsafe { core::intrinsics::volatile_load(ppte as *const u64) }
}

fn read_page_table_entry(va: usize) -> Option<u64> {
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

#[derive(Copy, Clone, Debug)]
pub struct EntryAttribute {
  pub executable: bool,
  pub writable: bool,
  pub copy_on_write: bool,
  pub shared: bool,
}

impl EntryAttribute {
  #[allow(dead_code)]
  pub const fn default() -> Self {
    EntryAttribute {
      executable: true,
      writable: true,
      copy_on_write: false,
      shared: false,
    }
  }
  #[allow(dead_code)]
  pub const fn executable() -> Self {
    EntryAttribute {
      executable: true,
      writable: false,
      copy_on_write: false,
      shared: false,
    }
  }
  #[allow(dead_code)]
  pub const fn writable() -> Self {
    EntryAttribute {
      executable: false,
      writable: true,
      copy_on_write: false,
      shared: false,
    }
  }
  #[allow(dead_code)]
  pub const fn copy_on_write() -> Self {
    EntryAttribute {
      executable: false,
      writable: false,
      copy_on_write: true,
      shared: false,
    }
  }
  #[allow(dead_code)]
  pub const fn shared() -> Self {
    EntryAttribute {
      executable: false,
      writable: false,
      copy_on_write: false,
      shared: true,
    }
  }
}

impl core::ops::Add<EntryAttribute> for EntryAttribute {
  type Output = EntryAttribute;

  fn add(self, rhs: EntryAttribute) -> Self::Output {
    EntryAttribute {
      executable: self.executable || rhs.executable,
      writable: self.writable || rhs.writable,
      copy_on_write: self.copy_on_write || rhs.copy_on_write,
      shared: self.shared || rhs.shared,
    }
  }
}


impl core::ops::Sub<EntryAttribute> for EntryAttribute {
  type Output = EntryAttribute;

  fn sub(self, rhs: EntryAttribute) -> Self::Output {
    EntryAttribute {
      executable: self.executable && !rhs.executable,
      writable: self.writable && !rhs.writable,
      copy_on_write: self.copy_on_write && !rhs.copy_on_write,
      shared: self.shared && !rhs.shared,
    }
  }
}

#[derive(Clone, Copy, Debug)]
pub struct ArchEntryAttribute(u64);

impl ArchEntryAttribute {
  pub fn new(value: u64) -> Self { ArchEntryAttribute(value) }
  pub fn to_usize(&self) -> usize { self.0 as usize }
}

impl core::convert::From<EntryAttribute> for ArchEntryAttribute {
  fn from(pte: EntryAttribute) -> Self {
    ArchEntryAttribute(
      (PAGE_DESCRIPTOR::USER::True
        + PAGE_DESCRIPTOR::R::True
        + if pte.writable {
        PAGE_DESCRIPTOR::W::True
      } else {
        PAGE_DESCRIPTOR::W::False
      } + if pte.executable {
        PAGE_DESCRIPTOR::X::True
      } else {
        PAGE_DESCRIPTOR::X::False
      } + if pte.copy_on_write {
        PAGE_DESCRIPTOR::COW::True
      } else {
        PAGE_DESCRIPTOR::COW::False
      } + if pte.shared {
        PAGE_DESCRIPTOR::LIB::True
      } else {
        PAGE_DESCRIPTOR::LIB::False
      }).value
    )
  }
}

impl core::convert::From<ArchEntryAttribute> for EntryAttribute {
  fn from(apte: ArchEntryAttribute) -> Self {
    use register::*;
    let reg = LocalRegisterCopy::<u64, PAGE_DESCRIPTOR::Register>::new(apte.0);
    EntryAttribute {
      executable: reg.is_set(PAGE_DESCRIPTOR::X),
      writable: reg.is_set(PAGE_DESCRIPTOR::W),
      copy_on_write: reg.is_set(PAGE_DESCRIPTOR::COW),
      shared: reg.is_set(PAGE_DESCRIPTOR::LIB),
    }
  }
}

#[allow(dead_code)]
pub const PTE_DEFAULT: EntryAttribute = EntryAttribute::default();
#[allow(dead_code)]
pub const PTE_X: EntryAttribute = EntryAttribute::executable();
#[allow(dead_code)]
pub const PTE_W: EntryAttribute = EntryAttribute::writable();
#[allow(dead_code)]
pub const PTE_COW: EntryAttribute = EntryAttribute::copy_on_write();
#[allow(dead_code)]
pub const PTE_LIB: EntryAttribute = EntryAttribute::shared();

pub fn query(va: usize) -> Option<EntryAttribute> {
  if let Some(pte) = read_page_table_entry(va) {
    Some(EntryAttribute::from(ArchEntryAttribute(pte)))
  } else {
    None
  }
}

pub fn traverse<F>(limit: usize, f: F) where F: Fn(usize, EntryAttribute) -> () {
  for l1x in 0..(PAGE_SIZE / WORD_SIZE) {
    let l1e = read_directory_entry(l1x);
    if l1e & 0b1 == 0 || l1e & (1 << 4) == 0 {
      continue;
    }
    for l2x in 0..(PAGE_SIZE / WORD_SIZE) {
      let va = (l1x << PAGE_TABLE_L1_SHIFT) + (l2x << PAGE_TABLE_L2_SHIFT);
      if va >= limit {
        return;
      }
      let l2e = read_level_1_entry(l1x, l2x);
      if l2e & 0b1 == 0 {
        continue;
      }
      for l3x in 0..(PAGE_SIZE / WORD_SIZE) {
        let va = (l1x << PAGE_TABLE_L1_SHIFT) + (l2x << PAGE_TABLE_L2_SHIFT) + (l3x << PAGE_TABLE_L3_SHIFT);
        if va >= limit {
          return;
        }
        let l3e = read_level_2_entry(l1x, l2x, l3x);
        if l3e & 0b1 != 0 {
          f(va, EntryAttribute::from(ArchEntryAttribute(l3e)));
        }
      }
    }
  }
}