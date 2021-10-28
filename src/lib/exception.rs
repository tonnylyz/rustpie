use crate::lib::thread::thread_destroy;
use crate::arch::ContextFrame;
use unwind::unwind_from_exception;
use crate::lib::cpu::cpu;
use crate::lib::traits::ContextFrameTrait;
use crate::util::round_up;
use core::mem::size_of;

pub fn handle_user() {
  match crate::lib::cpu::cpu().running_thread() {
    None => { panic!("isr: default: no running thread") }
    Some(t) => {
      match t.address_space() {
        None => {}
        Some(a) => {
          if a.asid() == 1 {
            // trusted exception
            if let Some(handler) = a.exception_handler() {
              let ctx = cpu().context_mut();
              let ctx_copied = *ctx;
                ctx.set_exception_pc(handler);
              let sp = ctx.stack_pointer() - round_up(size_of::<ContextFrame>(), 16);
              ctx.set_stack_pointer(sp);
              unsafe {
                // TODO: make it safe to write user mem
                (sp as *mut ContextFrame).write(ctx_copied);
              }
              ctx.set_argument(sp);
              return;
            } else {
              error!("trusted exception handler not found");
              thread_destroy(t);
              crate::lib::cpu::cpu().schedule();
            }
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
  unwind_from_exception(reg);
}
