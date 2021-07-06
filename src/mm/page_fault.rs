use common::CONFIG_USER_LIMIT;

use crate::arch::PAGE_SIZE;
use crate::core_id;
use crate::lib::cpu::cpu;
use crate::lib::traits::*;
use crate::util::*;
use crate::lib::thread::thread_destroy;

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
  trace!("thread t{} core {} page fault {:x}", t.tid(), core_id(), va);
  if va >= CONFIG_USER_LIMIT {
    warn!("isr: page_fault: {:016x} >= CONFIG_USER_LIMIT, process killed", va);
    thread_destroy(t);
    current_cpu().schedule();
    return;
  }

  warn!("isr: page_fault: {:016x}", addr);
  warn!("isr: page_fault: process has no handler, process killed");
  thread_destroy(t);
  current_cpu().schedule();
}