use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;

use spin::Mutex;

use crate::arch::{ContextFrame, CoreId};
use crate::lib::address_space::AddressSpace;
use crate::lib::bitmap::BitMap;
use crate::lib::cpu::CoreTrait;
use core::mem::size_of;
use crate::lib::traits::*;
use crate::lib::thread::Status::{TsRunnable, TsNotRunnable};

pub type Tid = u16;

#[derive(Debug)]
pub enum Type {
  User(AddressSpace),
  Kernel,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Status {
  TsRunnable = 1,
  TsNotRunnable = 2,
  TsIdle = 3,
  TsWaitForInterrupt = 4,
  TsWaitForThreadExit = 5,
}

#[derive(Debug)]
struct Inner {
  tid: Tid,
  parent: Option<Thread>,
  t: Type,
  status: Mutex<Status>,
  context_frame: Mutex<ContextFrame>,
  running_core: Mutex<Option<CoreId>>,
  itc_peer: Mutex<Option<Thread>>,
}

pub enum Error {
  ThreadNotFoundError
}

#[derive(Debug, Clone)]
pub struct Thread(Arc<Inner>);

impl PartialEq for Thread {
  fn eq(&self, other: &Self) -> bool {
    self.0.tid == other.0.tid
  }
}

impl Thread {
  pub fn tid(&self) -> Tid {
    self.0.tid
  }

  pub fn is_child_of(&self, tid: Tid) -> bool {
    match &self.0.parent {
      None => { false }
      Some(t) => {
        t.tid() == tid
      }
    }
  }

  pub fn status(&self) -> Status {
    *self.0.status.lock()
  }

  pub fn set_status(&self, status: Status) {
    let mut lock = self.0.status.lock();
    *lock = status;
  }

  pub fn runnable(&self) -> bool {
    let lock = self.0.status.lock();
    let r = *lock == Status::TsRunnable;
    r
  }

  pub fn address_space(&self) -> Option<AddressSpace> {
    match &self.0.t {
      Type::User(p) => {
        Some(p.clone())
      }
      _ => {
        None
      }
    }
  }

  pub fn set_context(&self, ctx: ContextFrame) {
    let mut lock = self.0.context_frame.lock();
    *lock = ctx;
  }

  pub fn context(&self) -> ContextFrame {
    let lock = self.0.context_frame.lock();
    (*lock).clone()
  }

  pub fn assign_to_current_core(&self) -> bool {
    let mut running_core = self.0.running_core.lock();
    match *running_core {
      None => {
        *running_core = Some(crate::core_id());
        true
      }
      Some(prev) => { prev == crate::core_id() }
    }
  }

  pub fn destroy(&self) {
    if let Some(t) = crate::current_cpu().running_thread() {
      if self.0.tid == t.tid() {
        crate::current_cpu().set_running_thread(None);
      }
    }
    free(self)
  }

  pub fn peer(&self) -> Option<Thread> {
    let ptr = self.0.itc_peer.lock();
    ptr.clone()
  }

  pub fn set_peer(&self, sender: Thread) {
    let mut ptr = self.0.itc_peer.lock();
    *ptr = Some(sender);
  }

  pub fn clear_peer(&self) {
    let mut ptr = self.0.itc_peer.lock();
    *ptr = None;
  }

  pub fn wake(&self) {
    self.set_status(TsRunnable);
  }

  pub fn sleep(&self) {
    self.set_status(TsNotRunnable);
  }

  pub fn receivable(&self, sender: &Thread) -> bool {
    (if let Some(peer) = self.peer() {
      peer.tid() == sender.tid()
    } else {
      true
    })&& self.status() == TsNotRunnable
  }

}

struct ThreadPool {
  bitmap: BitMap<{ Tid::MAX as usize / size_of::<usize>() }>,
  allocated: Vec<Thread>,
}

impl ThreadPool {
  fn alloc_user(&mut self, pc: usize, sp: usize, arg: usize, a: AddressSpace, t: Option<Thread>) -> Thread {
    let id = (self.bitmap.alloc() + 1) as Tid;
    let arc = Arc::new(Inner {
      tid: id,
      parent: t,
      t: Type::User(a),
      status: Mutex::new(Status::TsNotRunnable),
      context_frame: Mutex::new(ContextFrame::new(pc, sp, arg, false)),
      running_core: Mutex::new(None),
      itc_peer: Mutex::new(None)
    });
    let mut map = THREAD_MAP.get().unwrap().lock();
    map.insert(id, arc.clone());
    self.allocated.push(Thread(arc.clone()));
    Thread(arc)
  }

  fn alloc_kernel(&mut self, pc: usize, sp: usize, arg: usize) -> Thread {
    let id = (self.bitmap.alloc() + 1) as Tid;
    let arc = Arc::new(Inner {
      tid: id,
      parent: None,
      t: Type::Kernel,
      status: Mutex::new(Status::TsNotRunnable),
      context_frame: Mutex::new(ContextFrame::new(pc, sp, arg, true)),
      running_core: Mutex::new(None),
      itc_peer: Mutex::new(None)
    });
    let mut map = THREAD_MAP.get().unwrap().lock();
    map.insert(id, arc.clone());
    self.allocated.push(Thread(arc.clone()));
    Thread(arc)
  }

  fn free(&mut self, t: &Thread) -> Result<(), Error> {
    if self.allocated.contains(t) {
      self.allocated.retain(|_t| _t.tid() != t.tid());
      let mut map = THREAD_MAP.get().unwrap().lock();
      map.remove(&t.tid());
      self.bitmap.clear((t.tid() - 1) as usize);
      Ok(())
    } else {
      Err(Error::ThreadNotFoundError)
    }
  }

  fn list(&self) -> Vec<Thread> {
    self.allocated.clone()
  }
}

static THREAD_MAP: spin::Once<Mutex<BTreeMap<Tid, Arc<Inner>>>> = spin::Once::new();

pub fn init() {
  THREAD_MAP.call_once(|| {
    Mutex::new(BTreeMap::new())
  });
}

static THREAD_POOL: Mutex<ThreadPool> = Mutex::new(ThreadPool {
  bitmap: BitMap::new(),
  allocated: Vec::new(),
});

pub fn new_user(pc: usize, sp: usize, arg: usize, a: AddressSpace, t: Option<Thread>) -> Thread {
  let mut pool = THREAD_POOL.lock();
  let r = pool.alloc_user(pc, sp, arg, a, t);
  r
}

pub fn new_kernel(pc: usize, sp: usize, arg: usize) -> Thread {
  let mut pool = THREAD_POOL.lock();
  let r = pool.alloc_kernel(pc, sp, arg);
  r
}

pub fn free(t: &Thread) {
  let mut pool = THREAD_POOL.lock();
  match pool.free(t) {
    Ok(_) => {}
    Err(_) => { error!("thread_pool: free: thread not found") }
  }
}

pub fn list() -> Vec<Thread> {
  let pool = THREAD_POOL.lock();
  let r = pool.list();
  r
}

pub fn lookup(tid: Tid) -> Option<Thread> {
  let map = THREAD_MAP.get().unwrap().lock();
  let r = match map.get(&tid) {
    Some(arc) => Some(Thread(arc.clone())),
    None => None
  };
  r
}
