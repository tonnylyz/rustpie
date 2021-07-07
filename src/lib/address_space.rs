use alloc::collections::BTreeMap;
use alloc::sync::Arc;

use spin::Mutex;

use crate::arch::PageTable;
use crate::mm::page_table::{PageTableTrait, EntryAttribute, PageTableEntryAttrTrait};
use crate::util::round_up;
use common::{PAGE_SIZE, CONFIG_ELF_IMAGE};
use crate::lib::traits::Address;
use core::sync::atomic::{AtomicU16, Ordering};

pub type Asid = u16;

#[derive(Debug)]
struct Inner {
  asid: Asid,
  page_table: PageTable,
}

impl Drop for Inner {
  fn drop(&mut self) {
    info!("Drop AS{}", self.asid);
  }
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

fn make_user_page_table() -> PageTable {
  let frame = crate::mm::page_pool::alloc();
  let page_table = PageTable::new(frame);
  page_table.recursive_map(common::CONFIG_RECURSIVE_PAGE_TABLE_BTM);
  page_table
}

static ASID_ALLOCATOR: AtomicU16 = AtomicU16::new(1);

fn new_asid() -> Asid {
  ASID_ALLOCATOR.fetch_add(1, Ordering::Relaxed)
}

static ADDRESS_SPACE_MAP: Mutex<BTreeMap<Asid, AddressSpace>> = Mutex::new(BTreeMap::new());

pub fn address_space_alloc() -> Option<AddressSpace> {
  let id = new_asid();
  if id == 0 {
    return None;
  }
  let a = AddressSpace(Arc::new(Inner {
    asid: id,
    page_table: make_user_page_table(),
  }));
  let mut map = ADDRESS_SPACE_MAP.lock();
  map.insert(id, a.clone());
  Some(a)
}

pub fn address_space_lookup(asid: Asid) -> Option<AddressSpace> {
  let map = ADDRESS_SPACE_MAP.lock();
  match map.get(&asid) {
    Some(a) => Some(a).cloned(),
    None => None
  }
}

pub fn address_space_destroy(a: AddressSpace) {
  info!("Destroy AS{}", a.asid());
  let mut map = ADDRESS_SPACE_MAP.lock();
  map.remove(&a.asid());
}

pub fn load_image(elf: &'static [u8]) -> (AddressSpace, usize) {
  let a = address_space_alloc().unwrap();
  let page_table = a.page_table();
  let len = round_up(elf.len(), PAGE_SIZE);
  for i in (0..len).step_by(PAGE_SIZE) {
    let pa = (elf.as_ptr() as usize + i).kva2pa();
    page_table.map(CONFIG_ELF_IMAGE + i, pa, EntryAttribute::user_readonly());
  }
  match crate::lib::elf::load(elf, page_table) {
    Ok(entry) => {
      (a, entry)
    }
    Err(_) => { panic!("process: create: load err") }
  }
}
