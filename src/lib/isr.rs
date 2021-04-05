use crate::arch::{ContextFrameTrait, ArchTrait};
use crate::lib::core::{CoreTrait, current};
use crate::lib::syscall::{SystemCall, SystemCallTrait, SystemCallValue};
use crate::lib::page_table::PageTableTrait;

pub trait InterruptServiceRoutine {
  fn system_call();
  fn timer_interrupt();
  fn external_interrupt();
  fn page_fault();
  fn default();
}

pub struct Isr;

impl InterruptServiceRoutine for Isr {
  fn system_call() {
    let ctx = crate::lib::core::current().context_mut();
    let arg = |i: usize| { ctx.syscall_argument(i) };
    let scr = match ctx.syscall_number() {
      1 => {
        // SystemCall::null()
        SystemCall::putc(arg(0) as u8 as char)
      }
      2 => {
        SystemCall::get_asid(arg(0) as u16)
      }
      3 => {
        SystemCall::thread_yield()
      }
      4 => {
        SystemCall::address_space_destroy(arg(0) as u16)
      }
      5 => {
        SystemCall::process_set_exception_handler(arg(0) as u16, arg(1), arg(2))
      }
      6 => {
        SystemCall::mem_alloc(arg(0) as u16, arg(1), arg(2))
      }
      7 => {
        SystemCall::mem_map(arg(0) as u16, arg(1), arg(2) as u16, arg(3), arg(4))
      }
      8 => {
        SystemCall::mem_unmap(arg(0) as u16, arg(1))
      }
      9 => {
        SystemCall::address_space_alloc()
      }
      10 => {
        use crate::lib::thread::Status::{TsNotRunnable, TsRunnable};
        match arg(1) {
          1 => { SystemCall::thread_set_status(arg(0) as u16, TsRunnable) }
          2 => { SystemCall::thread_set_status(arg(0) as u16, TsNotRunnable) }
          _ => { Err(super::syscall::Error::InvalidArgumentError) }
        }
      }
      11 => {
        SystemCall::ipc_receive(arg(0))
      }
      12 => {
        SystemCall::ipc_can_send(arg(0) as u16, arg(1), arg(2), arg(3))
      }
      13 => {
        SystemCall::thread_alloc(arg(0), arg(1), arg(2))
      }
      _ => {
        println!("system call: unrecognized system call number");
        Err(super::syscall::Error::InvalidArgumentError)
      }
    };
    match scr {
      // SystemCallResult::Void => {}
      // SystemCallResult::Pid(pid) => {
      //   println!("syscall {}:{:?}", (*ctx).syscall_number(), scr);
      //   ctx.set_syscall_return_value(pid as usize);
      // }
      // SystemCallResult::R(o) => {
      //   println!("syscall {}:{:?}", (*ctx).syscall_number(), scr);
      //   match o {
      //     None => { ctx.set_syscall_return_value(0); }
      //     Some(i) => { ctx.set_syscall_return_value(i as usize); }
      //   }
      // }
      Ok(val) => {
        let num = (*ctx).syscall_number();
        if num != 1 {
          println!("syscall_{} Ok {:x?}", num, val);
        }

        match val {
          SystemCallValue::Unit => {}
          SystemCallValue::U32(u) => {
            ctx.set_syscall_return_value(u as usize);
          }
          SystemCallValue::U16(u) => {
            ctx.set_syscall_return_value(u as usize);
          }
          SystemCallValue::ISize(i) => {
            ctx.set_syscall_return_value(i as usize);
          }
          SystemCallValue::USize(u) => {
            ctx.set_syscall_return_value(u as usize);
          }
        }
      }
      Err(err) => {
        println!("syscall_{} Err {:x?}", (*ctx).syscall_number(), err);
        ctx.set_syscall_return_value(usize::MAX);
      }
    }
  }

  fn timer_interrupt() {
    crate::driver::timer::next();
    crate::lib::core::current().schedule();
  }

  fn external_interrupt() {
    panic!("external_interrupt");
  }

  fn page_fault() {
    let t = crate::lib::core::current().running_thread();
    if t.is_none() {
      panic!("isr: page_fault: no running thread");
    }
    let ctx = current().context();
    let fa = crate::arch::Arch::fault_address();
    println!("fault address {:016x}", fa);
    println!("{}", ctx);

    let a = t.unwrap().address_space().unwrap();
    let entry = a.page_table().lookup_page(fa);
    println!("{:?}", entry);


    panic!("");
    // let addr = Arch::fault_address();
    // let va = round_down(addr, PAGE_SIZE);
    // if va >= CONFIG_USER_LIMIT {
    //   println!("isr: page_fault: va >= CONFIG_USER_LIMIT, process killed");
    //   p.destroy();
    //   return;
    // }
    // if p.exception_handler().is_none() {
    //   println!("isr: page_fault: process has no handler, process killed");
    //   p.destroy();
    //   return;
    // }
    // let (entry, stack_top) = p.exception_handler().unwrap();
    // let page_table = p.page_table();
    // let stack_btm = stack_top - PAGE_SIZE;
    // match page_table.lookup_page(stack_btm) {
    //   Some(stack_pte) => {
    //     if va == stack_btm {
    //       println!("isr: page_fault: fault on exception stack, process killed");
    //       p.destroy();
    //       return;
    //     }
    //     let ctx = current_core().context_mut();
    //
    //     let stack_frame = PageFrame::new(stack_pte.pa());
    //     unsafe {
    //       memcpy(
    //         (stack_frame.kva() + PAGE_SIZE - size_of::<ContextFrame>()) as *mut ContextFrame,
    //         ctx as *mut ContextFrame,
    //         1,
    //       );
    //       ctx.set_exception_pc(entry);
    //       ctx.set_stack_pointer(stack_top - size_of::<ContextFrame>());
    //       ctx.set_argument(va);
    //     }
    //     return;
    //   }
    //   None => {
    //     println!("isr: page_fault: exception stack not valid, process killed");
    //     p.destroy();
    //     return;
    //   }
    // }
  }

  fn default() {
    match crate::lib::core::current().running_thread() {
      None => { panic!("isr: default: no running thread") }
      Some(t) => {
        t.destroy();
      }
    }
  }
}
