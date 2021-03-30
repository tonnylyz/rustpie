use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;

use spin::Mutex;

use crate::arch::{PAGE_SIZE, PageTable};
use crate::config::CONFIG_USER_STACK_TOP;
use crate::lib::bitmap::BitMap;
use crate::lib::current_thread;
use crate::lib::page_table::{EntryAttribute, PageTableEntryAttrTrait, PageTableTrait};
use crate::lib::thread::Thread;

pub type Asid = u16;

#[derive(Debug)]
pub struct ControlBlock {
  asid: Asid,
  threads: Mutex<Vec<Thread>>,
  parent: Option<AddressSpace>,
  page_table: PageTable,
  exception_handler: Mutex<Option<(usize, usize)>>,
}


#[derive(Debug, Clone)]
pub struct AddressSpace(Arc<ControlBlock>);

impl PartialEq for AddressSpace {
  fn eq(&self, other: &Self) -> bool {
    self.0.asid == other.0.asid
  }
}

impl AddressSpace {
  pub fn asid(&self) -> Asid {
    self.0.asid
  }

  pub fn main_thread(&self) -> Thread {
    let lock = self.0.threads.lock();
    let r = lock[0].clone();
    drop(lock);
    r
  }

  pub fn set_main_thread(&self, t: Thread) {
    let mut lock = self.0.threads.lock();
    assert!(lock.is_empty());
    lock.push(t);
    drop(lock);
  }

  pub fn exception_handler(&self) -> Option<(usize, usize)> {
    let lock = self.0.exception_handler.lock();
    let r = *lock;
    drop(lock);
    r
  }

  pub fn set_exception_handler(&self, entry: usize, stack_top: usize) {
    let mut lock = self.0.exception_handler.lock();
    *lock = Some((entry, stack_top));
    drop(lock);
  }

  pub fn page_table(&self) -> PageTable {
    self.0.page_table
  }

  pub fn parent(&self) -> Option<AddressSpace> {
    match &self.0.parent {
      None => { None }
      Some(p) => { Some(p.clone()) }
    }
  }

  pub fn destroy(&self) {
    for t in self.0.threads.lock().iter() {
      t.destroy();
    }
    self.0.page_table.destroy();
    let frame = self.0.page_table.directory();
    crate::mm::page_pool::decrease_rc(frame);
    free(self);
    if current_thread().is_none() {
      crate::lib::scheduler::schedule();
    }
  }
}


struct Pool {
  bitmap: BitMap,
  alloced: Vec<AddressSpace>,
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
  fn alloc(&mut self, parent: Option<AddressSpace>) -> AddressSpace {
    let id = (self.bitmap.alloc() + 1) as Asid;
    let arc = Arc::new(ControlBlock {
      asid: id,
      threads: Mutex::new(Vec::new()),
      parent,
      page_table: make_user_page_table(),
      exception_handler: Mutex::new(None),
    });
    let mut map = ADDRESS_SPACE_MAP.get().unwrap().lock();
    map.insert(id, arc.clone());
    drop(map);
    self.alloced.push(AddressSpace(arc.clone()));
    AddressSpace(arc)
  }

  fn free(&mut self, p: &AddressSpace) -> Result<(), Error> {
    if self.alloced.contains(p) {
      self.alloced.retain(|_p| _p.asid() != p.asid());
      let mut map = ADDRESS_SPACE_MAP.get().unwrap().lock();
      map.remove(&p.asid());
      drop(map);
      self.bitmap.clear((p.asid() - 1) as usize);
      Ok(())
    } else {
      Err(Error::AddressSpaceNotFound)
    }
  }

  #[allow(dead_code)]
  fn list(&self) -> Vec<AddressSpace> {
    self.alloced.clone()
  }
}

static ADDRESS_SPACE_MAP: spin::Once<Mutex<BTreeMap<Asid, Arc<ControlBlock>>>> = spin::Once::new();

pub fn init() {
  ADDRESS_SPACE_MAP.call_once(|| {
    Mutex::new(BTreeMap::new())
  });
}

static POOL: Mutex<Pool> = Mutex::new(Pool {
  bitmap: BitMap::new(),
  alloced: Vec::new(),
});

pub fn alloc(parent: Option<AddressSpace>) -> AddressSpace {
  let mut pool = POOL.lock();
  let r = pool.alloc(parent);
  drop(pool);
  r
}

pub fn free(p: &AddressSpace) {
  let mut pool = POOL.lock();
  match pool.free(p) {
    Ok(_) => {}
    Err(_) => { println!("process_pool: free: process not found") }
  }
  drop(pool);
}

#[allow(dead_code)]
pub fn list() -> Vec<AddressSpace> {
  let pool = POOL.lock();
  let r = pool.list();
  drop(pool);
  r
}

pub fn lookup(pid: Asid) -> Option<AddressSpace> {
  let map = ADDRESS_SPACE_MAP.get().unwrap().lock();
  let r = match map.get(&pid) {
    Some(arc) => Some(AddressSpace(arc.clone())),
    None => None
  };
  drop(map);
  r
}

pub fn create(elf: &'static [u8], arg: usize) {
  let p = alloc(None);
  let page_table = p.page_table();
  match crate::lib::elf::load(elf, page_table) {
    Ok(pc) => {
      let sp = CONFIG_USER_STACK_TOP;
      match page_table.insert_page(sp - PAGE_SIZE, crate::mm::page_pool::alloc(), EntryAttribute::user_default()) {
        Ok(_) => {}
        Err(_) => { panic!("process: create: page_table.insert_page failed") }
      }
      let t = crate::lib::thread::alloc_user(pc, sp, arg, p.clone());
      t.set_status(crate::lib::thread::Status::TsRunnable);
      p.set_main_thread(t);
    }
    Err(_) => { panic!("process: create: load err") }
  }
}