use spin::Once;

use crate::arch::{AddressSpaceId, ContextFrame, PAGE_SIZE};
use crate::board::BOARD_CORE_NUMBER;
use crate::core_id;
use crate::lib::address_space::AddressSpace;
use crate::mm::page_table::PageTableTrait;
use crate::lib::scheduler::{RoundRobinScheduler, SchedulerTrait};
use crate::lib::thread::Thread;
use crate::mm::PageFrame;
use crate::lib::traits::*;

pub trait CoreTrait {
  fn context(&self) -> &ContextFrame;
  fn context_mut(&self) -> &mut ContextFrame;

  fn set_context(&mut self, ctx: *mut ContextFrame);
  fn clear_context(&mut self);

  fn has_context(&self) -> bool;
  fn running_thread(&self) -> Option<Thread>;
  fn set_running_thread(&mut self, p: Option<Thread>);

  fn schedule(&mut self);

  fn create_idle_thread(&self);
  fn idle_thread(&self) -> Thread;
  fn run(&mut self, t: Thread);

  fn address_space(&self) -> Option<AddressSpace>;
  fn set_address_space(&mut self, a: AddressSpace);
  fn clear_address_space(&mut self);
}

pub struct Core {
  context: Option<*mut ContextFrame>,
  // pointer points at stack
  running_thread: Option<Thread>,
  scheduler: RoundRobinScheduler,
  idle_thread: Once<Thread>,
  idle_stack: Once<PageFrame>,
  address_space: Option<AddressSpace>,
}

// Note: only the core itself can be allowed to access its `Core`
unsafe impl core::marker::Send for Core {}

unsafe impl core::marker::Sync for Core {}

const CORE: Core = Core {
  context: None,
  running_thread: None,
  scheduler: RoundRobinScheduler::new(),
  idle_thread: Once::new(),
  idle_stack: Once::new(),
  address_space: None,
};

static mut CORES: [Core; BOARD_CORE_NUMBER] = [CORE; BOARD_CORE_NUMBER];

impl CoreTrait for Core {
  fn context(&self) -> &ContextFrame {
    unsafe { self.context.unwrap().as_ref() }.unwrap()
  }

  fn context_mut(&self) -> &mut ContextFrame {
    unsafe { self.context.unwrap().as_mut() }.unwrap()
  }

  fn set_context(&mut self, ctx: *mut ContextFrame) {
    self.context = Some(ctx);
  }

  fn clear_context(&mut self) {
    self.context = None;
  }

  fn has_context(&self) -> bool {
    self.context.is_some()
  }

  fn running_thread(&self) -> Option<Thread> {
    self.running_thread.clone()
  }

  fn set_running_thread(&mut self, t: Option<Thread>) {
    self.running_thread = t
  }

  fn schedule(&mut self) {
    let t = self.scheduler.schedule();
    match t {
      None => {
        self.run(self.idle_thread())
      }
      Some(t) => {
        // println!("[CORE{}] run t{}", core_id(), t.tid());
        self.run(t);
      }
    }
  }

  fn create_idle_thread(&self) {
    let frame = crate::mm::page_pool::alloc();
    let t = crate::lib::thread::new_kernel(
      idle_thread as usize,
      frame.kva() + PAGE_SIZE,
      crate::core_id());
    t.set_status(crate::lib::thread::Status::TsIdle);
    self.idle_thread.call_once(|| t);
    self.idle_stack.call_once(|| frame);
  }

  fn idle_thread(&self) -> Thread {
    self.idle_thread.get().unwrap().clone()
  }

  fn run(&mut self, t: Thread) {
    if let Some(prev) = self.running_thread() {
      // Note: normal switch
      prev.set_context(*self.context());

      *self.context_mut() = t.context();
    } else {
      if self.has_context() {
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

  fn address_space(&self) -> Option<AddressSpace> {
    self.address_space.clone()
  }

  fn set_address_space(&mut self, a: AddressSpace) {
    let mut prev_id = u16::MAX;
    if let Some(prev) = &self.address_space {
      prev_id = prev.asid();
    }
    if a.asid() != prev_id {
      self.address_space = Some(a.clone());
      crate::arch::PageTable::set_user_page_table(a.page_table().base_pa(), a.asid() as AddressSpaceId);
      crate::arch::Arch::invalidate_tlb();
    }
  }

  fn clear_address_space(&mut self) {
    self.address_space = None
  }
}

pub fn current() -> &'static mut Core {
  let core_id = crate::core_id();
  unsafe { &mut CORES[core_id] }
}

#[no_mangle]
fn idle_thread(_arg: usize) {
  loop {
    crate::arch::Arch::wait_for_interrupt();
  }
}
