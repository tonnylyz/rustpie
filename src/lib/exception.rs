use crate::lib::thread::thread_destroy;
use crate::arch::{ContextFrame, PAGE_SIZE};
use unwind::unwind_from_exception;
use crate::lib::cpu::cpu;
use crate::lib::traits::ContextFrameTrait;
use crate::lib::traits::ArchTrait;
use crate::util::{round_down, round_up};
use core::mem::size_of;
use crate::mm::Frame;
use crate::mm::page_table::{PageTableEntryAttrTrait, PageTableTrait};

enum HandleResult {
  Ok, // thread can continue running
  Kill(&'static str), // thread need to be killed
  Err(&'static str), // system state corrupt (something goes very wrong)
}

fn handle() -> HandleResult {
  if let Some(t) = crate::lib::cpu::cpu().running_thread() {
    if let Some(a) = t.address_space() {
      if a.asid() == 1 {
        // trusted exception
        if let Some(handler) = a.exception_handler() {
          let ctx = cpu().context_mut();
          let ctx_copied = *ctx;
          ctx.set_exception_pc(handler);
          let sp = ctx.stack_pointer() - round_up(size_of::<ContextFrame>(), 16);
          ctx.set_stack_pointer(sp);
          let sp_va = round_down(sp, PAGE_SIZE);
          let pt = a.page_table();
          if let None = pt.lookup_user_page(sp_va) {
            if let Ok(frame) = crate::mm::page_pool::page_alloc() {
              if let Err(_) = pt.insert_page(sp_va, crate::mm::Frame::from(frame),
                                             crate::mm::page_table::EntryAttribute::user_default()) {
                return HandleResult::Err("page insert failed")
              }
            } else {
              thread_destroy(t);
              return HandleResult::Kill("out of memory")
            }
          }
          unsafe {
            (sp as *mut ContextFrame).write(ctx_copied);
          }
          ctx.set_argument(sp);
          HandleResult::Ok
        } else {
          thread_destroy(t);
          HandleResult::Kill("trusted has no handler")
        }
      } else {
        thread_destroy(t);
        HandleResult::Kill("user program exception")
      }
    } else {
      HandleResult::Err("running thread has no address space")
    }
  } else {
    HandleResult::Err("no running thread")
  }
}

pub fn handle_user() {
  match handle() {
    HandleResult::Ok => {}
    HandleResult::Kill(e) => {
      warn!("handle user {}", e);
      crate::lib::cpu::cpu().schedule();
    }
    HandleResult::Err(e) => panic!("handle user {}", e),
  }
}

pub fn handle_kernel(ctx: &ContextFrame, is_page_fault: bool) {
  if is_page_fault {
    error!("kernel page fault exception occurs {:016x}", crate::arch::Arch::fault_address());
  } else {
    error!("kernel other exception occurs");
  }
  let ctx = ctx.clone();
  let reg = ctx.into();
  unwind_from_exception(reg);
}
