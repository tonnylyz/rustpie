use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::mem::size_of;

use spin::Mutex;

use crate::arch::PageTable;
use crate::lib::bitmap::BitMap;
use crate::mm::page_table::{PageTableTrait, EntryAttribute, PageTableEntryAttrTrait};
use crate::round_up;
use common::{PAGE_SIZE, CONFIG_ELF_IMAGE};
use crate::lib::traits::Address;

pub type Asid = u16;

#[derive(Debug)]
struct Inner {
  asid: Asid,
  page_table: PageTable,
}

#[derive(Debug, Clone)]
pub struct AddressSpace(Arc<Inner>);

impl PartialEq for AddressSpace {
  fn eq(&self, other: &Self) -> bool {
    self.0.asid == other.0.asid
  }
}

impl AddressSpace {
  pub fn asid(&self) -> Asid {
    self.0.asid
  }
  pub fn page_table(&self) -> &PageTable {
    &self.0.page_table
  }

}

struct Pool {
  bitmap: BitMap<{ Asid::MAX as usize / size_of::<usize>() }>,
  allocated: Vec<AddressSpace>,
}

fn make_user_page_table() -> PageTable {
  let frame = crate::mm::page_pool::alloc();
  let page_table = PageTable::new(frame);
  page_table.recursive_map(common::CONFIG_RECURSIVE_PAGE_TABLE_BTM);
  page_table
}

impl Pool {
  fn alloc(&mut self) -> AddressSpace {
    let id = (self.bitmap.alloc() + 1) as Asid;
    let arc = Arc::new(Inner {
      asid: id,
      page_table: make_user_page_table(),
    });
    let mut map = ADDRESS_SPACE_MAP.get().unwrap().lock();
    map.insert(id, arc.clone());
    self.allocated.push(AddressSpace(arc.clone()));
    AddressSpace(arc)
  }

}

static ADDRESS_SPACE_MAP: spin::Once<Mutex<BTreeMap<Asid, Arc<Inner>>>> = spin::Once::new();

pub fn init() {
  ADDRESS_SPACE_MAP.call_once(|| {
    Mutex::new(BTreeMap::new())
  });
}

static POOL: Mutex<Pool> = Mutex::new(Pool {
  bitmap: BitMap::new(),
  allocated: Vec::new(),
});

pub fn alloc() -> AddressSpace {
  let mut pool = POOL.lock();
  let r = pool.alloc();
  r
}

pub fn lookup(asid: Asid) -> Option<AddressSpace> {
  let map = ADDRESS_SPACE_MAP.get().unwrap().lock();
  let r = match map.get(&asid) {
    Some(arc) => Some(AddressSpace(arc.clone())),
    None => None
  };
  r
}

pub fn load_image(elf: &'static [u8]) -> (AddressSpace, usize) {
  let a = alloc();
  let page_table = a.page_table();
  let va_start = elf.as_ptr() as usize;
  let len = round_up(elf.len(), PAGE_SIZE);
  for i in (0..len).step_by(PAGE_SIZE) {
    let pa = (va_start + i).kva2pa();
    page_table.map(CONFIG_ELF_IMAGE + i, pa, EntryAttribute::user_readonly());
  }
  match crate::lib::elf::load(elf, page_table) {
    Ok(entry) => {
      (a, entry)
    }
    Err(_) => { panic!("process: create: load err") }
  }
}
