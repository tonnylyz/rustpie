use common::{CONFIG_USER_LIMIT, CONFIG_USER_STACK_BTM, CONFIG_USER_STACK_TOP};

use crate::arch::PAGE_SIZE;
use crate::lib::cpu::cpu;
use crate::lib::traits::*;
use crate::util::*;
use crate::lib::thread::thread_destroy;
use crate::mm::page_table::{PageTableTrait, PageTableEntryAttrTrait};

pub fn handle() {
  let t = cpu().running_thread();
  if t.is_none() {
    panic!("isr: page_fault: no running thread");
  }
  let t = t.unwrap();
  if t.address_space().is_none() {
    panic!("isr: kernel thread page fault");
  }
  let addr = crate::arch::Arch::fault_address();
  let va = round_down(addr, PAGE_SIZE);
  trace!("thread t{} core {} page fault {:x}", t.tid(), crate::arch::Arch::core_id(), va);
  if va >= CONFIG_USER_LIMIT {
    warn!("isr: page_fault: {:016x} >= CONFIG_USER_LIMIT, process killed", va);
    thread_destroy(t);
    crate::lib::cpu::cpu().schedule();
    return;
  }

  warn!("isr: page_fault: addr {:016x} pc {:016x}", addr, cpu().context().exception_pc());
  // NOTE: allocate stack region automatically
  if addr > CONFIG_USER_STACK_BTM && addr < CONFIG_USER_STACK_TOP {
    let a = t.address_space().unwrap();
    let pt = a.page_table();
    if pt.lookup_page(va).is_none() {
      let frame = crate::mm::page_pool::page_alloc();
      if let Ok(frame) = frame {
        pt.insert_page(va, crate::mm::Frame::from(frame),
                       crate::mm::page_table::EntryAttribute::user_default()).map_err(|_e| {
          warn!("insert page failed");
        });
      } else {
        warn!("oom");
      }
      return;
    } else {
      warn!("page table entry exists")
    }
  }
  warn!("isr: page_fault: process has no handler, process killed");
  thread_destroy(t);
  crate::lib::cpu::cpu().schedule();
}