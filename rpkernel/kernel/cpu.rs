use spin::Once;
use spin::Mutex;
use alloc::collections::VecDeque as RunQueue;

use crate::MAX_CPU_NUMBER;
use crate::arch::{AddressSpaceId, ContextFrame, PAGE_SIZE};
use crate::core_id;
use crate::kernel::address_space::AddressSpace;
use crate::kernel::scheduler::scheduler;
use crate::kernel::thread::Thread;
use crate::kernel::traits::*;
use crate::mm::PhysicalFrame;

pub struct Core {
  context: Option<*mut ContextFrame>,
  // pointer points at stack
  running_thread: Option<Thread>,
  running_idle: bool,
  run_queue: Mutex<RunQueue<Thread>>,
  idle_thread: Once<Thread>,
  idle_stack: Once<PhysicalFrame>,
  address_space: Option<AddressSpace>,
}

// Note: only the core itself can be allowed to access its `Core`
unsafe impl core::marker::Send for Core {}

unsafe impl core::marker::Sync for Core {}

const CORE: Core = Core {
  context: None,
  running_thread: None,
  running_idle: false,
  run_queue: Mutex::new(RunQueue::new()),
  idle_thread: Once::new(),
  idle_stack: Once::new(),
  address_space: None,
};

static mut CORES: [Core; MAX_CPU_NUMBER] = [CORE; MAX_CPU_NUMBER];

impl Core {
  // context

  pub fn context(&self) -> &ContextFrame {
    unsafe { self.context.unwrap().as_ref() }.unwrap()
  }

  pub fn context_mut(&self) -> &mut ContextFrame {
    unsafe { self.context.unwrap().as_mut() }.unwrap()
  }

  pub fn set_context(&mut self, ctx: *mut ContextFrame) {
    self.context = Some(ctx);
  }

  pub fn clear_context(&mut self) {
    self.context = None;
  }

  // thread

  pub fn running_thread(&self) -> Option<Thread> {
    self.running_thread.clone()
  }

  pub fn set_running_thread(&mut self, t: Option<Thread>) {
    self.running_thread = t
  }

  fn idle_thread(&self) -> Thread {
    match self.idle_thread.get() {
      None => {
        let frame = crate::mm::page_pool::page_alloc().expect("fail to allocate idle thread stack");
        let t = crate::kernel::thread::new_kernel(
          idle_thread as usize,
          frame.kva() + PAGE_SIZE,
          0);
        self.idle_stack.call_once(|| frame);
        self.idle_thread.call_once(|| t).clone()
      }
      Some(t) => t.clone(),
    }
  }

  pub fn running_idle(&self) -> bool {
    self.running_idle
  }

  pub fn enqueue_task(&self, t: Thread, to_front: bool) -> bool {
    let mut run_queue = self.run_queue.lock();
    let r = run_queue.is_empty();
    if to_front {
      run_queue.push_front(t);
    } else {
      run_queue.push_back(t);
    }
    r
  }

  pub fn tick(&mut self, queue_prev: bool) {
    let mut run_queue = self.run_queue.lock();
    if let Some(next) = run_queue.pop_front() {
      self.running_idle = false;
      drop(run_queue);
      self.run(next, queue_prev);
    } else {
      self.running_idle = true;
      drop(run_queue);
      self.run(self.idle_thread(), queue_prev);
    }
    crate::driver::timer::next();
  }

  fn run(&mut self, t: Thread, queue_prev: bool) {
    if let Some(prev) = self.running_thread() {
      // info!("t{}->[{}]", prev.tid(), t.tid());
      // Note: normal switch
      prev.set_context(*self.context());
      prev.clear_running_cpu();
      // add back to scheduler queue if it explicitly yield
      if prev.tid() != self.idle_thread().tid() && queue_prev {
        scheduler().add(prev.clone());
      }
      *self.context_mut() = t.context();
    } else {
      if self.context.is_some() {
        // Note: previous process has been destroyed
        *self.context_mut() = t.context();
      } else {
        // Note: this is first run
        // `main` prepare the context to stack
      }
    }
    self.set_running_thread(Some(t.clone()));
    t.set_running_cpu(core_id());
    if let Some(a) = t.address_space() {
      self.set_address_space(a);
    }
  }

  pub fn address_space(&self) -> Option<AddressSpace> {
    self.address_space.clone()
  }

  fn set_address_space(&mut self, a: AddressSpace) {
    if let Some(prev) = &self.address_space {
      if prev.asid() == a.asid() {
        return;
      }
      // info!("switch as from {} to {}", prev.asid(), a.asid());
    }
    self.address_space = Some(a.clone());
    crate::arch::Arch::install_user_page_table(a.page_table().directory_pa(), a.asid() as AddressSpaceId);
  }
}

pub fn cpu() -> &'static mut Core {
  let core_id = core_id();
  unsafe { &mut CORES[core_id] }
}

pub fn cpu_nth(core_id: usize) -> &'static Core {
  unsafe { &CORES[core_id] }
}

#[no_mangle]
fn idle_thread(_arg: usize) {
  loop {
    crate::arch::Arch::wait_for_interrupt();
  }
}
