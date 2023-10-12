use alloc::string::String;

use rpabi::syscall::*;
use rpabi::syscall::error::{ERROR_HOLD_ON, ERROR_INVARG};

use crate::kernel::cpu::cpu;
use crate::kernel::traits::ContextFrameTrait;

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
  "yield_to",
  "reply_recv",
];


static SYSCALL_ARGC: [usize; SYS_MAX] = [
  1, 1, 1, 0, 0, 1, 2, 3, 5, 2, 0, 4, 2, 1, 0, 5, 5, 1, 1, 1, 0, 1, 5
];

pub fn syscall() {
  use crate::syscall::*;

  let ctx = crate::kernel::cpu::cpu().context_mut();
  let tid = cpu().running_thread().map(|x| { x.tid() }).unwrap_or_default();
  let arg = |i: usize| { ctx.syscall_argument(i) };
  let num = ctx.syscall_number();
  let result = match num {
    SYS_NULL => misc::null(),
    SYS_PUTC => misc::putc(arg(0) as u8 as char),
    SYS_GETC => misc::getc(),
    SYS_SET_EXCEPTION_HANDLER => misc::set_exception_handler(arg(0)),
    SYS_GET_ASID => address_space::get_asid(arg(0)),
    SYS_ADDRESS_SPACE_ALLOC => address_space::address_space_alloc(),
    SYS_ADDRESS_SPACE_DESTROY => address_space::address_space_destroy(arg(0) as u16),
    SYS_GET_TID => thread::get_tid(),
    SYS_THREAD_YIELD => thread::thread_yield(),
    SYS_THREAD_DESTROY => thread::thread_destroy(arg(0)),
    SYS_THREAD_ALLOC => thread::thread_alloc(arg(0) as u16, arg(1), arg(2), arg(3)),
    SYS_THREAD_SET_STATUS => thread::thread_set_status(arg(0), arg(1)),
    SYS_EVENT_WAIT => event::event_wait(arg(0), arg(1)),
    SYS_MEM_ALLOC => mm::mem_alloc(arg(0) as u16, arg(1), arg(2)),
    SYS_MEM_MAP => mm::mem_map(arg(0) as u16, arg(1), arg(2) as u16, arg(3), arg(4)),
    SYS_MEM_UNMAP => mm::mem_unmap(arg(0) as u16, arg(1)),
    SYS_ITC_RECV => ipc::itc_receive(),
    SYS_ITC_SEND => ipc::itc_send(arg(0), arg(1), arg(2), arg(3), arg(4)),
    SYS_ITC_CALL => ipc::itc_call(arg(0), arg(1), arg(2), arg(3), arg(4)),
    SYS_REPLY_RECV => ipc::itc_reply_recv(arg(0), arg(1), arg(2), arg(3), arg(4)),
    SYS_SERVER_REGISTER => server::server_register(arg(0)),
    SYS_SERVER_TID => server::server_tid(arg(0)),
    _ => {
      warn!("system call: unrecognized system call number");
      Err(ERROR_INVARG)
    }
  };
  match result {
    Ok(ref res) => {
      match num {
        SYS_PUTC | SYS_THREAD_YIELD | SYS_MEM_ALLOC => {},
        _ => {
          trace!("{} t{} Ok {}", SYSCALL_NAMES[num], tid, res.0);
        }
      }
      
      let need_schedule = res.1;
      if need_schedule {
        cpu().tick(num == SYS_THREAD_YIELD);
      } else {
        ctx.set_syscall_result(&res.0, 0);
      }
      
    }
    Err(err) => {
      if err != ERROR_HOLD_ON {
        let mut arg_str = String::from("(");
        for i in 0..(SYSCALL_ARGC[num]-1) {
          arg_str += format!("{:x},", arg(i)).as_str();
        }
        arg_str += format!("{:x},", arg(SYSCALL_ARGC[num]-1)).as_str();
        arg_str += ")";
        info!("{} t{} arg{} Err {:x?}", SYSCALL_NAMES[num], tid, arg_str, err);
      }
      ctx.set_syscall_result(&SyscallOutRegisters::Unit, err);
    }
  }
}
