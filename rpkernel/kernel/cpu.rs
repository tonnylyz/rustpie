use spin::Once;
use spin::Mutex;
use alloc::collections::VecDeque as RunQueue;

use crate::arch::{AddressSpaceId, ContextFrame, PAGE_SIZE};
use crate::board::BOARD_CORE_NUMBER;
use crate::kernel::address_space::AddressSpace;
use crate::kernel::scheduler::scheduler;
use crate::kernel::thread::Thread;
use crate::kernel::traits::*;
use crate::mm::page_table::PageTableTrait;
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

static mut CORES: [Core; BOARD_CORE_NUMBER] = [CORE; BOARD_CORE_NUMBER];

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
          crate::arch::Arch::core_id());
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

  pub fn tick(&mut self) {
    let mut run_queue = self.run_queue.lock();
    if let Some(next) = run_queue.pop_front() {
      self.running_idle = false;
      drop(run_queue);
      self.run(next);
    } else {
      self.running_idle = true;
      drop(run_queue);
      self.run(self.idle_thread());
    }
    crate::driver::timer::next();
  }

  fn run(&mut self, t: Thread) {
    if let Some(prev) = self.running_thread() {
      // info!("switch thread from {} to {}", prev.tid(), t.tid());
      // Note: normal switch
      prev.set_context(*self.context());
      // add back to scheduler queue
      if prev.runnable() {
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
    crate::arch::PageTable::install_user_page_table(a.page_table().base_pa(), a.asid() as AddressSpaceId);
  }
}

pub fn cpu() -> &'static mut Core {
  let core_id = crate::arch::Arch::core_id();
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
