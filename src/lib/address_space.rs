use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::sync::atomic::{AtomicU16, Ordering};

use rpabi::{CONFIG_ELF_IMAGE, PAGE_SIZE};
use rpabi::syscall::error::{ERROR_OOM, ERROR_OOR};
use spin::Mutex;

use crate::arch::PageTable;
use crate::lib::traits::Address;
use crate::mm::page_table::{EntryAttribute, PageTableEntryAttrTrait, PageTableTrait};
use crate::util::round_up;

pub type Asid = u16;
pub type Error = usize;

#[derive(Debug)]
struct Inner {
  asid: Asid,
  page_table: PageTable,
  exception_handler: Mutex<Option<usize>>,
}

impl Drop for Inner {
  fn drop(&mut self) {
    trace!("Drop AS{}", self.asid);
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

  pub fn exception_handler(&self) -> Option<usize> {
    let lock = self.0.exception_handler.lock();
    lock.clone()
  }

  pub fn set_exception_handler(&self, handler: Option<usize>) {
    let mut lock = self.0.exception_handler.lock();
    *lock = handler;
  }
}

static ASID_ALLOCATOR: AtomicU16 = AtomicU16::new(1);

fn new_asid() -> Asid {
  ASID_ALLOCATOR.fetch_add(1, Ordering::Relaxed)
}

static ADDRESS_SPACE_MAP: Mutex<BTreeMap<Asid, AddressSpace>> = Mutex::new(BTreeMap::new());

pub fn address_space_alloc() -> Result<AddressSpace, Error> {
  let id = new_asid();
  if id == 0 {
    return Err(ERROR_OOR);
  }
  let frame = crate::mm::page_pool::page_alloc().map_err(|_| ERROR_OOM)?;
  frame.zero();
  let page_table = PageTable::new(frame);
  page_table.recursive_map(rpabi::CONFIG_RECURSIVE_PAGE_TABLE_BTM);
  let a = AddressSpace(Arc::try_new(Inner {
    asid: id,
    page_table,
    exception_handler: Mutex::new(None),
  }).map_err(|_| ERROR_OOM)?);
  let mut map = ADDRESS_SPACE_MAP.lock();
  map.insert(id, a.clone());
  Ok(a)
}

pub fn address_space_lookup(asid: Asid) -> Option<AddressSpace> {
  let map = ADDRESS_SPACE_MAP.lock();
  match map.get(&asid) {
    Some(a) => Some(a).cloned(),
    None => None
  }
}

pub fn address_space_destroy(a: AddressSpace) {
  trace!("Destroy AS{}", a.asid());
  let mut map = ADDRESS_SPACE_MAP.lock();
  map.remove(&a.asid());
}

pub fn load_image(elf: &'static [u8]) -> (AddressSpace, usize) {
  // let icntr = crate::lib::timer::current_cycle();
  let a = address_space_alloc().unwrap();
  // let icntr2 = crate::lib::timer::current_cycle();
  // info!("as create cycle {}", icntr2 - icntr);
  let page_table = a.page_table();
  let len = round_up(elf.len(), PAGE_SIZE);
  for i in (0..len).step_by(PAGE_SIZE) {
    let pa = (elf.as_ptr() as usize + i).kva2pa();
    page_table.map(CONFIG_ELF_IMAGE + i, pa, EntryAttribute::user_readonly()).expect("page_table map error");
  }
  let entry = crate::lib::elf::load(elf, page_table).expect("elf load error");
  (a, entry)
}
