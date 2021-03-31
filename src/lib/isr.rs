use crate::arch::ContextFrameTrait;
use crate::lib::address_space::Asid;
use crate::lib::core::CoreTrait;
use crate::lib::syscall::{SystemCall, SystemCallTrait};

pub trait InterruptServiceRoutine {
  fn system_call();
  fn timer_interrupt();
  fn external_interrupt();
  fn page_fault();
  fn default();
}

pub struct Isr;

#[derive(Debug)]
pub enum SystemCallResult {
  Void,
  Pid(Asid),
  R(Option<isize>),
}

pub trait SystemCallResultOk {
  fn to_isize(&self) -> isize;
}

impl SystemCallResultOk for () {
  fn to_isize(&self) -> isize {
    0
  }
}

impl SystemCallResultOk for u16 {
  fn to_isize(&self) -> isize {
    self.clone() as isize
  }
}

impl core::convert::From<Asid> for SystemCallResult {
  fn from(pid: Asid) -> Self {
    SystemCallResult::Pid(pid)
  }
}

impl core::convert::From<()> for SystemCallResult {
  fn from(_: ()) -> Self {
    SystemCallResult::Void
  }
}

impl<T> core::convert::From<Result<T, crate::lib::syscall::Error>> for SystemCallResult where T: SystemCallResultOk {
  fn from(sce: Result<T, crate::lib::syscall::Error>) -> Self {
    SystemCallResult::R(
      match sce {
        Ok(t) => { Some(t.to_isize()) }
        Err(e) => { Some(-(e as isize)) }
      }
    )
  }
}

impl InterruptServiceRoutine for Isr {
  fn system_call() {
    let ctx = crate::lib::core::current().context_mut();
    let arg = |i: usize| { ctx.syscall_argument(i) };
    let scr = match ctx.syscall_number() {
      1 => {
        // print!("core_{}: putc({:02x})", crate::core_id(), arg(0) as u8);
        // println!();
        // ().into()
        SystemCall::putc(arg(0) as u8 as char).into()
      }
      2 => {
        SystemCall::get_asid().into()
      }
      3 => {
        SystemCall::thread_yield().into()
      }
      4 => {
        SystemCall::address_space_destroy(arg(0) as u16).into()
      }
      5 => {
        SystemCall::process_set_exception_handler(arg(0) as u16, arg(1), arg(2)).into()
      }
      6 => {
        SystemCall::mem_alloc(arg(0) as u16, arg(1), arg(2)).into()
      }
      7 => {
        SystemCall::mem_map(arg(0) as u16, arg(1), arg(2) as u16, arg(3), arg(4)).into()
      }
      8 => {
        SystemCall::mem_unmap(arg(0) as u16, arg(1)).into()
      }
      9 => {
        SystemCall::address_space_alloc().into()
      }
      10 => {
        use crate::lib::thread::Status::{TsNotRunnable, TsRunnable};
        match arg(1) {
          1 => { SystemCall::thread_set_status(arg(0) as u16, TsRunnable).into() }
          2 => { SystemCall::thread_set_status(arg(0) as u16, TsNotRunnable).into() }
          _ => { ().into() }
        }
      }
      11 => {
        SystemCall::ipc_receive(arg(0)).into()
      }
      12 => {
        SystemCall::ipc_can_send(arg(0) as u16, arg(1), arg(2), arg(3)).into()
      }
      13 => {
        SystemCall::thread_alloc(arg(0), arg(1), arg(2)).into()
      }
      _ => { println!("system call: unrecognized system call number").into() }
    };
    match scr {
      SystemCallResult::Void => {}
      SystemCallResult::Pid(pid) => {
        println!("syscall {}:{:?}", (*ctx).syscall_number(), scr);
        ctx.set_syscall_return_value(pid as usize);
      }
      SystemCallResult::R(o) => {
        println!("syscall {}:{:?}", (*ctx).syscall_number(), scr);
        match o {
          None => { ctx.set_syscall_return_value(0); }
          Some(i) => { ctx.set_syscall_return_value(i as usize); }
        }
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
    panic!();
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
    //       core::intrinsics::volatile_copy_memory(
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
