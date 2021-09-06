use crate::lib::thread::thread_destroy;
use crate::arch::ContextFrame;
use unwind::start_unwinding_from_exception;



pub fn handle_user() {
  match crate::lib::cpu::cpu().running_thread() {
    None => { panic!("isr: default: no running thread") }
    Some(t) => {
      match t.address_space() {
        None => {}
        Some(a) => {
          if a.asid() == 0 {
            // trusted exception
            // TODO: forward exception to userspace
          } else {
            thread_destroy(t);
            crate::lib::cpu::cpu().schedule();
          }
        }
      }
    }
  }
}

pub fn handle_kernel(ctx: &ContextFrame, is_page_fault: bool) {
  if is_page_fault {
    error!("kernel page fault exception occurs");
  } else {
    error!("kernel other exception occurs");
  }
  let ctx = ctx.clone();
  let reg = ctx.into();
  start_unwinding_from_exception(reg);
}
