use spin::Once;

use crate::arch::{AddressSpaceId, ContextFrame, PAGE_SIZE};
use crate::board::BOARD_CORE_NUMBER;
use crate::lib::address_space::AddressSpace;
use crate::lib::scheduler::scheduler;
use crate::lib::thread::Thread;
use crate::lib::traits::*;
use crate::mm::page_table::PageTableTrait;
use crate::mm::PhysicalFrame;

pub struct Core {
  context: Option<*mut ContextFrame>,
  // pointer points at stack
  running_thread: Option<Thread>,
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
        let t = crate::lib::thread::new_kernel(
          idle_thread as usize,
          frame.kva() + PAGE_SIZE,
          crate::arch::Arch::core_id());
        self.idle_stack.call_once(|| frame);
        self.idle_thread.call_once(|| t).clone()
      }
      Some(t) => t.clone(),
    }
  }

  pub fn schedule(&mut self) {
    if let Some(t) = scheduler().pop() {
      self.run(t);
    } else {
      self.run(self.idle_thread());
    }
  }

  pub fn schedule_to(&mut self, t: Thread) {
    self.run(t);
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

#[no_mangle]
fn idle_thread(_arg: usize) {
  loop {
    crate::arch::Arch::wait_for_interrupt();
  }
}
