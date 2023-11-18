#![no_std]

use rpabi::syscall::*;

use arch::*;

#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64.rs"]
mod arch;

#[cfg(target_arch = "riscv64")]
#[path = "arch/riscv64.rs"]
mod arch;

#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64.rs"]
mod arch;

pub fn null(dummy: usize) { let _ = syscall_1_0(SYS_NULL, dummy); }

pub fn putraw(c: u8) {
  let _ = syscall_1_0(SYS_PUTC, c as usize);
}

pub fn putc(c: char) {
  putraw(c as u8);
}

pub fn get_asid(tid: usize) -> Result<u16, Error> {
  syscall_1_1(SYS_GET_ASID, tid).map(|asid| asid as u16)
}

pub fn get_tid() -> usize {
  syscall_0_1(SYS_GET_TID).unwrap()
}

pub fn thread_yield() {
  let _ = syscall_0_0(SYS_THREAD_YIELD);
}

pub fn thread_destroy(tid: usize) {
  syscall_1_0(SYS_THREAD_DESTROY, tid).unwrap()
}

pub fn event_wait(event_type: usize, event_num: usize) -> Result<usize, Error> {
  syscall_2_1(SYS_EVENT_WAIT, event_type, event_num)
}

pub fn mem_alloc(asid: u16, va: usize, attr: usize) -> Result<(), Error> {
  syscall_3_0(SYS_MEM_ALLOC, asid as usize, va, attr)
}

pub fn mem_map(src_asid: u16, src_va: usize, dst_asid: u16, dst_va: usize, attr: usize) -> Result<(), Error> {
  syscall_5_0(SYS_MEM_MAP, src_asid as usize, src_va, dst_asid as usize, dst_va, attr)
}

pub fn mem_unmap(asid: u16, va: usize) -> Result<(), Error> {
  syscall_2_0(SYS_MEM_UNMAP, asid as usize, va)
}

pub fn address_space_alloc() -> Result<u16, Error> {
  syscall_0_1(SYS_ADDRESS_SPACE_ALLOC).map(|asid| asid as u16)
}

pub fn thread_alloc(asid: u16, entry: usize, sp: usize, arg: usize) -> Result<usize, Error> {
  syscall_4_1(SYS_THREAD_ALLOC, asid as usize, entry, sp, arg)
}

pub fn thread_set_status(tid: usize, status: usize) -> Result<(), Error> {
  syscall_2_0(SYS_THREAD_SET_STATUS, tid, status)
}

pub fn address_space_destroy(asid: u16) -> Result<(), Error> {
  syscall_1_0(SYS_ADDRESS_SPACE_DESTROY, asid as usize)
}

pub fn itc_receive() -> Result<(usize, usize, usize, usize, usize), Error> {
  syscall_0_5(SYS_ITC_RECV)
}

pub fn itc_send(tid: usize, a: usize, b: usize, c: usize, d: usize) -> Result<(), Error> {
  syscall_5_0(SYS_ITC_SEND, tid as usize, a, b, c, d)
}

pub fn itc_call(tid: usize, a: usize, b: usize, c: usize, d: usize) -> Result<(usize, usize, usize, usize, usize), Error> {
  syscall_5_5(SYS_ITC_CALL, tid as usize, a, b, c, d)
}

pub fn server_register(server_id: usize) -> Result<(), Error> {
  syscall_1_0(SYS_SERVER_REGISTER, server_id)
}

fn server_tid(server_id: usize) -> Result<usize, Error> {
  syscall_1_1(SYS_SERVER_TID, server_id)
}

pub fn server_tid_wait(server_id: usize) -> usize {
  loop {
    if let Ok(tid) = server_tid(server_id) {
      break tid;
    }
    thread_yield();
  }
}

pub fn set_exception_handler(handler: usize) -> Result<(), Error> {
  syscall_1_0(SYS_SET_EXCEPTION_HANDLER, handler)
}

pub fn getc() -> Result<u8, Error> {
  syscall_0_1(SYS_GETC).map(|c| c as u8)
}

pub fn itc_recv_reply(tid: usize, a: usize, b: usize, c: usize, d: usize) -> Result<(usize, usize, usize, usize, usize), Error> {
  syscall_5_5(SYS_REPLY_RECV, tid as usize, a, b, c, d)
}

pub mod message {

  #[repr(C)]
  #[derive(Copy, Clone, Debug, Default)]
  pub struct Message {
    pub a: usize,
    pub b: usize,
    pub c: usize,
    pub d: usize,
  }

  impl Message {
    pub fn new(a: usize, b: usize, c: usize, d: usize) -> Self {
      Message {
        a,
        b,
        c,
        d,
      }
    }

    pub fn receive() -> Result<(usize, Self), super::Error> {
      super::itc_receive().map(|(tid, a, b, c, d)|
        (tid, Message { a, b, c, d }))
    }

    pub fn send_to(&self, tid: usize) -> Result<(), super::Error> {
      super::itc_send(tid, self.a, self.b, self.c, self.d)
    }
    
    pub fn reply_recv(&self, tid: usize) -> Result<(usize, Self), super::Error> {
      super::itc_recv_reply(tid, self.a, self.b, self.c, self.d).map(|(tid, a, b, c, d)|
        (tid, Message { a, b, c, d }))
    }
    
    pub fn call(&self, server_id: usize) -> Result<Self, super::Error> {
      use rpabi::syscall::error::ERROR_HOLD_ON;
      let server_tid = super::server_tid_wait(server_id);
      loop {
        match super::itc_call(server_tid, self.a, self.b, self.c, self.d) {
          Ok((_, a, b, c, d)) => {
            break Ok(Message { a, b, c, d });
          }
          Err(ERROR_HOLD_ON) => {
            super::thread_yield();
          }
          Err(e) => {
            break Err(e);
          }
        }
      }
    }
  }
}