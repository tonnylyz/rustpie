use spin::Mutex;

use crate::arch::{ArchTrait, ContextFrame, CoreTrait, PAGE_SIZE};
use crate::board::BOARD_CORE_NUMBER;
use crate::lib::scheduler::{RoundRobinScheduler, SchedulerTrait};
use crate::lib::thread::Thread;
use crate::lib::round_up;

pub struct Core {
  context: Mutex<*mut ContextFrame>,
  running_thread: Mutex<Option<Thread>>,
  scheduler: Mutex<RoundRobinScheduler>,
  idle_thread: Mutex<Option<Thread>>,
}

const CORE_STACK_PAGE_NUM: usize = 127;

#[repr(align(4096))]
pub struct Stack {
  _protective_hole: [u8; PAGE_SIZE],
  stack: [u8; PAGE_SIZE * CORE_STACK_PAGE_NUM],
}

impl Stack {
  pub fn top(&self) -> usize {
    (&self.stack as *const _ as usize) + PAGE_SIZE * CORE_STACK_PAGE_NUM
  }
}

const STACK: Stack = Stack {
  _protective_hole: [0; PAGE_SIZE],
  stack: [0; PAGE_SIZE * CORE_STACK_PAGE_NUM]
};

#[no_mangle]
static STACKS: [Stack; BOARD_CORE_NUMBER] = [STACK; BOARD_CORE_NUMBER];

// Note: only the core itself can be allowed to access its `Core`
unsafe impl core::marker::Send for Core {}

unsafe impl core::marker::Sync for Core {}

const CORE: Core = Core {
  context: Mutex::new(0usize as *mut ContextFrame),
  running_thread: Mutex::new(None),
  scheduler: Mutex::new(RoundRobinScheduler::new()),
  idle_thread: Mutex::new(None),
};

static CORES: [Core; BOARD_CORE_NUMBER] = [CORE; BOARD_CORE_NUMBER];

impl CoreTrait for Core {
  fn context(&self) -> &ContextFrame {
    let lock = self.context.lock();
    let r = unsafe { (*lock).as_ref() }.unwrap();
    drop(lock);
    r
  }

  fn context_mut(&self) -> &mut ContextFrame {
    let lock = self.context.lock();
    let r = unsafe { (*lock).as_mut() }.unwrap();
    drop(lock);
    r
  }

  fn set_context(&self, ctx: *mut ContextFrame) {
    let mut lock = self.context.lock();
    *lock = ctx;
    drop(lock);
  }

  fn clear_context(&self) {
    let mut lock = self.context.lock();
    *lock = 0usize as *mut ContextFrame;
    drop(lock);
  }

  fn has_context(&self) -> bool {
    let lock = self.context.lock();
    let r = unsafe { (*lock).as_ref() }.is_some();
    drop(lock);
    r
  }

  fn running_thread(&self) -> Option<Thread> {
    let lock = self.running_thread.lock();
    let r = lock.clone();
    drop(lock);
    r
  }

  fn set_running_thread(&self, t: Option<Thread>) {
    let mut lock = self.running_thread.lock();
    *lock = t;
    drop(lock);
  }

  fn schedule(&self) {
    let mut lock = self.scheduler.lock();
    lock.schedule();
    drop(lock);
  }

  fn create_idle_thread(&self) {
    let t = crate::lib::thread::alloc_kernel(
      idle_thread as usize,
      crate::mm::page_pool::alloc().kva() + PAGE_SIZE,
      crate::arch::Arch::core_id());
    t.set_status(crate::lib::thread::Status::TsIdle);
    let mut lock = self.idle_thread.lock();
    *lock = Some(t);
    drop(lock);
  }

  fn idle_thread(&self) -> Thread {
    let lock = self.idle_thread.lock();
    let r = lock.as_ref().unwrap().clone();
    drop(lock);
    r
  }
}

pub fn current() -> &'static Core {
  let core_id = crate::arch::Arch::core_id();
  &CORES[core_id]
}

pub fn stack() -> &'static Stack {
  let core_id = crate::arch::Arch::core_id();
  &STACKS[core_id]
}

struct CoreBarrier {
  count: u32,
}
static CORE_BARRIER: Mutex<CoreBarrier> = Mutex::new(CoreBarrier { count: 0 });
pub fn barrier() {
  let next_count;
  let mut barrier = CORE_BARRIER.lock();
  barrier.count += 1;
  next_count = round_up(barrier.count as usize, BOARD_CORE_NUMBER);
  drop(barrier);
  loop {
    if CORE_BARRIER.lock().count as usize >= next_count {
      break;
    }
  }
}

fn idle_thread(_arg: usize) {
  loop {
    crate::arch::Arch::wait_for_event();
  }
}
