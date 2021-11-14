use alloc::string::String;

use common::syscall::*;
use common::syscall::error::{ERROR_HOLD_ON, ERROR_INVARG};

use unwind::catch::catch_unwind;

use crate::lib::cpu::cpu;
use crate::lib::traits::ContextFrameTrait;

static SYSCALL_NAMES: [&str; SYS_MAX] = [
  "null",
  "putc",
  "get_asid",
  "get_tid",
  "thread_yield",
  "thread_destroy",
  "event_wait",
  "mem_alloc",
  "mem_map",
  "mem_unmap",
  "address_space_alloc",
  "thread_alloc",
  "thread_set_status",
  "address_space_destroy",
  "itc_recv",
  "itc_send",
  "itc_call",
  "server_register",
  "server_tid",
  "set_exception_handler",
  "getc",
];


static SYSCALL_ARGC: [usize; SYS_MAX] = [
  0, 1, 1, 0, 0, 1, 2, 3, 5, 2, 0, 4, 2, 1, 0, 5, 5, 1, 1, 1, 0
];

pub fn syscall() {
  use crate::syscall::*;

  let ctx = crate::lib::cpu::cpu().context_mut();
  let tid = cpu().running_thread().map(|x| { x.tid() }).unwrap_or_default();
  let arg = |i: usize| { ctx.syscall_argument(i) };
  let num = ctx.syscall_number();
  let result = catch_unwind(|| {
    match num {
      SYS_NULL => misc::null(),
      SYS_PUTC => misc::putc(arg(0) as u8 as char),
      SYS_GET_ASID => address_space::get_asid(arg(0)),
      SYS_GET_TID => thread::get_tid(),
      SYS_THREAD_YIELD => thread::thread_yield(),
      SYS_THREAD_DESTROY => thread::thread_destroy(arg(0)),
      SYS_EVENT_WAIT => event::event_wait(arg(0), arg(1)),
      SYS_MEM_ALLOC => mm::mem_alloc(arg(0) as u16, arg(1), arg(2)),
      SYS_MEM_MAP => mm::mem_map(arg(0) as u16, arg(1), arg(2) as u16, arg(3), arg(4)),
      SYS_MEM_UNMAP => mm::mem_unmap(arg(0) as u16, arg(1)),
      SYS_ADDRESS_SPACE_ALLOC => address_space::address_space_alloc(),
      SYS_THREAD_ALLOC => thread::thread_alloc(arg(0) as u16, arg(1), arg(2), arg(3)),
      SYS_THREAD_SET_STATUS => thread::thread_set_status(arg(0), arg(1)),
      SYS_ADDRESS_SPACE_DESTROY => address_space::address_space_destroy(arg(0) as u16),
      SYS_ITC_RECV => ipc::itc_receive(),
      SYS_ITC_SEND => ipc::itc_send(arg(0), arg(1), arg(2), arg(3), arg(4)),
      SYS_ITC_CALL => ipc::itc_call(arg(0), arg(1), arg(2), arg(3), arg(4)),
      SYS_SERVER_REGISTER => server::server_register(arg(0)),
      SYS_SERVER_TID => server::server_tid(arg(0)),
      SYS_SET_EXCEPTION_HANDLER => misc::set_exception_handler(arg(0)),
      SYS_GETC => misc::getc(),
      _ => {
        warn!("system call: unrecognized system call number");
        Err(ERROR_INVARG)
      }
    }
  });
  match result {
    Ok(ref r) => match r {
      Ok(ref regs) => {
        if num != 1 {
          trace!("#{} {} t{} Ok {}", num, SYSCALL_NAMES[num], tid, regs);
        }
      }
      Err(err) => {
        if *err != ERROR_HOLD_ON {
          let mut arg_str = String::from("(");
          for i in 0..SYSCALL_ARGC[num] {
            arg_str += format!("{:x},", arg(i)).as_str();
          }
          arg_str += ")";
          info!("#{} {} t{} arg{} Err {:x?}", num, SYSCALL_NAMES[num], tid, arg_str, err);
        }
      }
    }
    Err(_) => {
      trace!("#{} {} t{} Panic", num, SYSCALL_NAMES[num], tid);
    }
  }


  if tid == cpu().running_thread().map(|x| { x.tid() }).unwrap_or_default() {
    match result {
      Ok(ref r) => { ctx.set_syscall_result(r); }
      Err(_) => { ctx.set_syscall_result(&Err(common::syscall::error::ERROR_PANIC)) }
    }
  }
}
