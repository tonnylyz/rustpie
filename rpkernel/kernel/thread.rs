use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::Relaxed;

use spin::Mutex;

use crate::arch::ContextFrame;
use crate::kernel::address_space::AddressSpace;
use crate::kernel::scheduler::scheduler;
use crate::kernel::traits::*;
use crate::syscall::event::thread_exit_signal;

pub type Tid = usize;

#[derive(Debug)]
pub enum PrivilegedLevel {
  User,
  Kernel,
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Status {
  Runnable,
  Sleep,
  WaitForEvent,
  WaitForReply,
  WaitForRequest,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Inner {
  uuid: usize,
  parent: Option<usize>,
  level: PrivilegedLevel,
  address_space: Option<AddressSpace>,
}

struct InnerMut {
  status: Mutex<Status>,
  context_frame: Mutex<ContextFrame>,
  running_cpu: Mutex<Option<usize>>,
}

struct ControlBlock {
  inner: Inner,
  inner_mut: InnerMut,
}

impl Drop for ControlBlock {
  fn drop(&mut self) {
    trace!("Drop t{}", self.inner.uuid);
  }
}

#[derive(Clone)]
pub struct Thread(Arc<ControlBlock>);

impl Thread {
  pub fn tid(&self) -> Tid {
    self.0.inner.uuid
  }

  pub fn parent(&self) -> Option<Tid> {
    self.0.inner.parent
  }

  pub fn is_child_of(&self, tid: Tid) -> bool {
    match &self.0.inner.parent {
      None => { false }
      Some(t) => {
        *t == tid
      }
    }
  }

  pub fn running_cpu(&self) -> Option<usize> {
    let cpu = self.0.inner_mut.running_cpu.lock();
    cpu.clone()
  }

  pub fn set_running_cpu(&self, new: usize) {
    let mut cpu = self.0.inner_mut.running_cpu.lock();
    assert!(cpu.is_none());
    *cpu = Some(new);
  }

  pub fn clear_running_cpu(&self) {
    let mut cpu = self.0.inner_mut.running_cpu.lock();
    *cpu = None;
  }

  pub fn status(&self) -> Status {
    let lock = self.0.inner_mut.status.lock();
    lock.clone()
  }

  pub fn wait_for_reply<F>(&self, f: F) -> bool where F: FnOnce() {
    let mut status = self.0.inner_mut.status.lock();
    if *status == Status::WaitForReply {
      f();
      *status = Status::Runnable;
      scheduler().add(self.clone());
      true
    } else {
      false
    }
  }

  pub fn wait_for_request<F>(&self, f: F) -> bool where F: FnOnce() {
    let mut status = self.0.inner_mut.status.lock();
    if *status == Status::WaitForRequest {
      f();
      *status = Status::Runnable;
      scheduler().add(self.clone());
      true
    } else {
      false
    }
  }

  pub fn address_space(&self) -> Option<AddressSpace> {
    self.0.inner.address_space.clone()
  }

  pub fn set_context(&self, ctx: ContextFrame) {
    let mut context_frame = self.0.inner_mut.context_frame.lock();
    *context_frame = ctx;
  }

  pub fn context(&self) -> ContextFrame {
    let lock = self.0.inner_mut.context_frame.lock();
    lock.clone()
  }

  pub fn map_with_context<F, T>(&self, f: F) -> T where F: FnOnce(&mut ContextFrame) -> T {
    loop {
      if self.running_cpu().is_none() {
        break;
      }
    }
    let mut context_frame = self.0.inner_mut.context_frame.lock();
    f(&mut *context_frame)
  }
}

static THREAD_UUID_ALLOCATOR: AtomicUsize = AtomicUsize::new(100);

fn new_tid() -> Tid {
  THREAD_UUID_ALLOCATOR.fetch_add(1, Relaxed)
}

static THREAD_MAP: Mutex<BTreeMap<Tid, Thread>> = Mutex::new(BTreeMap::new());

pub fn new_user(pc: usize, sp: usize, arg: usize, a: AddressSpace, parent: Option<Tid>) -> Thread {
  let id = new_tid();
  let t = Thread(Arc::new(ControlBlock {
    inner: Inner {
      uuid: id,
      parent,
      level: PrivilegedLevel::User,
      address_space: Some(a),
    },
    inner_mut: InnerMut {
      status: Mutex::new(Status::Sleep),
      context_frame: Mutex::new(ContextFrame::new(pc, sp, arg, false)),
      running_cpu: Mutex::new(None),
    },
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
      address_space: None,
    },
    inner_mut: InnerMut {
      status: Mutex::new(Status::Sleep),
      context_frame: Mutex::new(ContextFrame::new(pc, sp, arg, true)),
      running_cpu: Mutex::new(None),
    },
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
  trace!("Destroy t{}", t.tid());
  if let Some(current_thread) = crate::kernel::cpu::cpu().running_thread() {
    if t.tid() == current_thread.tid() {
      crate::kernel::cpu::cpu().set_running_thread(None);
    }
  }
  if let Some(parent) = t.parent() {
    thread_exit_signal(t.tid(), parent);
  }
  let mut map = THREAD_MAP.lock();
  map.remove(&t.tid());
}

pub fn thread_wake(t: &Thread) {
  let mut status = t.0.inner_mut.status.lock();
  *status = Status::Runnable;
  scheduler().add(t.clone());
}

pub fn thread_sleep(t: &Thread, reason: Status) {
  assert_ne!(reason, Status::Runnable);
  let mut status = t.0.inner_mut.status.lock();
  *status = reason;
  drop(status);
}
