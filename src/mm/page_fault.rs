use crate::arch::{PAGE_SIZE, ContextFrame};
use crate::lib::cpu::{CoreTrait, current};
use crate::lib::traits::*;
use crate::lib::event::Event;
use crate::util::*;
use common::CONFIG_USER_LIMIT;
use crate::mm::page_table::PageTableTrait;

use core::mem::size_of;
use crate::core_id;

pub fn handle() {
  let t = current().running_thread();
  if t.is_none() {
    panic!("isr: page_fault: no running thread");
  }
  let t = t.unwrap();
  if t.address_space().is_none() {
    panic!("isr: kernel thread page fault");
  }
  let p = t.address_space().unwrap();

  let addr = crate::arch::Arch::fault_address();
  let va = round_down(addr, PAGE_SIZE);
  trace!("thread t{} core {} page fault {:x}", t.tid(), core_id(), va);
  if va >= CONFIG_USER_LIMIT {
    warn!("isr: page_fault: {:016x} >= CONFIG_USER_LIMIT, process killed", va);
    t.destroy();
    current_cpu().schedule();
    return;
  }
  if p.event_handler(Event::PageFault).is_none() {
    warn!("isr: page_fault: {:016x}", addr);
    warn!("isr: page_fault: process has no handler, process killed");
    t.destroy();
    current_cpu().schedule();
    return;
  }
  let (entry, stack_top) = p.event_handler(Event::PageFault).unwrap();
  let page_table = p.page_table();
  let stack_btm = stack_top - PAGE_SIZE;
  match page_table.lookup_user_page(stack_btm) {
    Some(uf) => {
      if va == stack_btm {
        warn!("isr: page_fault: fault on exception stack, process killed");
        t.destroy();
        current_cpu().schedule();
        return;
      }
      let ctx = current().context_mut();
      unsafe {
        ((uf.pa().pa2kva() + PAGE_SIZE - size_of::<ContextFrame>()) as *mut ContextFrame)
          .write_volatile(*ctx);
      }
      ctx.set_exception_pc(entry);
      ctx.set_stack_pointer(stack_top - size_of::<ContextFrame>());
      ctx.set_argument(va);
      return;
    }
    None => {
      warn!("isr: page_fault: exception stack not valid, process killed");
      t.destroy();
      current_cpu().schedule();
      return;
    }
  }
}