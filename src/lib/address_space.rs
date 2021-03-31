use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;

use spin::Mutex;

use crate::arch::PageTable;
use crate::lib::bitmap::BitMap;
use crate::lib::page_table::PageTableTrait;

pub type Asid = u16;

#[derive(Debug)]
pub struct Inner {
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
  pub fn page_table(&self) -> PageTable {
    self.0.page_table
  }
  pub fn destroy(&self) {
    self.0.page_table.destroy();
    let frame = self.0.page_table.directory();
    crate::mm::page_pool::decrease_rc(frame);
    free(self);
  }
}

struct Pool {
  bitmap: BitMap,
  allocated: Vec<AddressSpace>,
}

pub enum Error {
  AddressSpaceNotFound,
}

fn make_user_page_table() -> PageTable {
  let frame = crate::mm::page_pool::alloc();
  crate::mm::page_pool::increase_rc(frame);
  let page_table = PageTable::new(frame);
  page_table.recursive_map(crate::config::CONFIG_RECURSIVE_PAGE_TABLE_BTM);
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

  fn free(&mut self, p: &AddressSpace) -> Result<(), Error> {
    if self.allocated.contains(p) {
      self.allocated.retain(|_p| _p.asid() != p.asid());
      let mut map = ADDRESS_SPACE_MAP.get().unwrap().lock();
      map.remove(&p.asid());
      self.bitmap.clear((p.asid() - 1) as usize);
      Ok(())
    } else {
      Err(Error::AddressSpaceNotFound)
    }
  }

  #[allow(dead_code)]
  fn list(&self) -> Vec<AddressSpace> {
    self.allocated.clone()
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

pub fn free(p: &AddressSpace) {
  let mut pool = POOL.lock();
  match pool.free(p) {
    Ok(_) => {}
    Err(_) => { println!("process_pool: free: process not found") }
  }
}

// #[allow(dead_code)]
// pub fn list() -> Vec<AddressSpace> {
//   let pool = POOL.lock();
//   let r = pool.list();
//   r
// }

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
  match crate::lib::elf::load(elf, page_table) {
    Ok(entry) => {
      (a, entry)
    }
    Err(_) => { panic!("process: create: load err") }
  }
}
