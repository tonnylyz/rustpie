use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use alloc::vec::Vec;
use core::mem::size_of;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::Relaxed;

use spin::{Mutex, Once};

use crate::arch::ContextFrame;
use crate::lib::address_space::AddressSpace;
use crate::lib::bitmap::BitMap;
use crate::lib::event::thread_exit_signal;
use crate::lib::scheduler::scheduler;
use crate::lib::traits::*;

pub type Tid = usize;

#[derive(Debug)]
pub enum PrivilegedLevel {
  User,
  Kernel,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Status {
  Runnable,
  Sleep,
}

#[derive(Debug)]
struct Inner {
  uuid: usize,
  parent: Option<usize>,
  level: PrivilegedLevel,
  address_space: Option<AddressSpace>,
}

struct InnerMut {
  status: Status,
  context_frame: ContextFrame,
  receiving: bool,
  caller: Option<Tid>,
}

struct ControlBlock {
  inner: Inner,
  inner_mut: Mutex<InnerMut>,
}

#[derive(Clone)]
pub struct Thread(Arc<ControlBlock>);

impl Thread {
  pub fn tid(&self) -> Tid {
    self.0.inner.uuid
  }

  pub fn is_child_of(&self, tid: Tid) -> bool {
    match &self.0.inner.parent {
      None => { false }
      Some(t) => {
        *t == tid
      }
    }
  }

  pub fn runnable(&self) -> bool {
    let lock = self.0.inner_mut.lock();
    lock.status == Status::Runnable
  }

  pub fn address_space(&self) -> Option<AddressSpace> {
    self.0.inner.address_space.clone()
  }

  pub fn set_context(&self, ctx: ContextFrame) {
    let mut lock = self.0.inner_mut.lock();
    lock.context_frame = ctx;
  }

  pub fn context(&self) -> ContextFrame {
    let mut lock = self.0.inner_mut.lock();
    lock.context_frame.clone()
  }

  pub fn map_with_context<F, T>(&self, f: F) -> T where F: FnOnce(&mut ContextFrame) -> T {
    let mut lock = self.0.inner_mut.lock();
    f(&mut lock.deref_mut().context_frame)
  }

  pub fn serve(&self, caller: Tid) -> bool {
    let mut lock = self.0.inner_mut.lock();
    if lock.caller.is_some() || lock.receiving == false {
      false
    } else {
      lock.receiving = false;
      lock.caller = Some(caller);
      true
    }
  }

  pub fn is_serving(&self) -> Option<Tid> {
    let mut lock = self.0.inner_mut.lock();
    lock.caller.clone()
  }

  pub fn ready_to_serve(&self) {
    let mut lock = self.0.inner_mut.lock();
    lock.caller = None;
  }

  pub fn receive(&self) -> bool {
    let mut lock = self.0.inner_mut.lock();
    if lock.receiving {
      lock.receiving = false;
      true
    } else {
      false
    }
  }

  pub fn ready_to_receive(&self) {
    let mut lock = self.0.inner_mut.lock();
    lock.receiving = true;
  }

  pub fn sleep(&self) {
    thread_sleep(self);
  }

  pub fn wake(&self) {
    thread_wake(self);
  }
}

static THREAD_UUID_ALLOCATOR: AtomicUsize = AtomicUsize::new(100);

fn new_tid() -> Tid {
  THREAD_UUID_ALLOCATOR.fetch_add(1, Relaxed)
}

static THREAD_MAP: Mutex<BTreeMap<Tid, Thread>> = Mutex::new(BTreeMap::new());

pub fn init() {}

pub fn new_user(pc: usize, sp: usize, arg: usize, a: AddressSpace, parent: Option<Tid>) -> Thread {
  let id = new_tid();
  let t = Thread(Arc::new(ControlBlock {
    inner: Inner {
      uuid: id,
      parent,
      level: PrivilegedLevel::User,
      address_space: Some(a)
    },
    inner_mut: Mutex::new(InnerMut {
      status: Status::Sleep,
      context_frame: ContextFrame::new(pc, sp, arg, false),
      receiving: false,
      caller: Some(0)
    })
  }));
  let mut map = THREAD_MAP.lock();
  map.insert(id, t.clone());
  t
}

pub fn new_kernel(pc: usize, sp: usize, arg: usize) -> Thread {
  let id = new_tid();
  let t = Thread(Arc::new(ControlBlock {
    inner: Inner {
      uuid: id,
      parent: None,
      level: PrivilegedLevel::Kernel,
      address_space: None
    },
    inner_mut: Mutex::new(InnerMut {
      status: Status::Sleep,
      context_frame: ContextFrame::new(pc, sp, arg, true),
      receiving: false,
      caller: None
    })
  }));
  let mut map = THREAD_MAP.lock();
  map.insert(id, t.clone());
  t
}

pub fn thread_lookup(tid: Tid) -> Option<Thread> {
  let map = THREAD_MAP.lock();
  map.get(&tid).cloned()
}

pub fn thread_destroy(t: Thread) {
  if let Some(current_thread) = crate::current_cpu().running_thread() {
    if t.tid() == current_thread.tid() {
      crate::current_cpu().set_running_thread(None);
    }
  }
  thread_exit_signal(t.tid());
  let mut map = THREAD_MAP.lock();
  map.remove(&t.tid());
}

pub fn thread_wake(t: &Thread) {
  let mut lock = t.0.inner_mut.lock();
  lock.status = Status::Runnable;
  scheduler().add(t.clone());
}

pub fn thread_sleep(t: &Thread) {
  let mut lock = t.0.inner_mut.lock();
  lock.status = Status::Sleep;
}
