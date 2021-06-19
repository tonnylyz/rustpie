#![no_std]
#![feature(asm)]
#![feature(global_asm)]

#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64/mod.rs"]
mod arch;

// #[cfg(target_arch = "riscv64")]
// #[path = "arch/riscv64/mod.rs"]
// mod arch;

use arch::*;
use common::syscall::*;

pub fn null() { syscall_0_0(SYS_NULL).unwrap() }

pub fn putc(c: char) {
  syscall_1_0(SYS_PUTC, c as usize).unwrap()
}

pub fn get_asid() -> u16 {
  syscall_0_1(SYS_GET_ASID).unwrap() as u16
}

pub fn get_tid() -> u16 {
  syscall_0_1(SYS_GET_TID).unwrap() as u16
}

pub fn thread_yield() {
  syscall_0_0(SYS_THREAD_YIELD);
}

pub fn thread_destroy(tid: u16) -> Result<(), Error> {
  syscall_1_0(SYS_THREAD_DESTROY, tid as usize)
}

pub fn event_handler(asid: u16, value: usize, sp: usize, event: usize) -> Result<(), Error> {
  syscall_4_0(SYS_EVENT_HANDLER, asid as usize, value, sp, event)
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

#[inline(always)]
pub fn address_space_alloc() -> Result<u16, Error> {
  syscall_0_1(SYS_ADDRESS_SPACE_ALLOC).map(|asid| asid as u16)
}

pub fn thread_alloc(asid: u16, entry: usize, sp: usize, arg: usize) -> Result<u16, Error> {
  syscall_4_1(SYS_THREAD_ALLOC, asid as usize, entry, sp, arg).map(|tid| tid as u16)
}

pub fn thread_set_status(tid: u16, status: usize) -> Result<(), Error> {
  syscall_2_0(SYS_THREAD_SET_STATUS, tid as usize, status)
}

pub fn ipc_receive() {
  todo!()
}

pub fn ipc_can_send() {
  todo!()
}

pub fn itc_receive() -> Result<(u16, usize, usize, usize, usize), Error> {
  syscall_0_5(SYS_ITC_RECV).map(|(tid, a, b, c, d)| (tid as u16, a, b, c, d))
}

pub fn itc_send(tid: u16, a: usize, b: usize, c: usize, d: usize) -> Result<(), Error> {
  syscall_5_0(SYS_ITC_SEND, tid as usize, a, b, c, d)
}

pub fn itc_call(tid: u16, a: usize, b: usize, c: usize, d: usize) -> Result<(u16, usize, usize, usize, usize), Error> {
  syscall_5_5(SYS_ITC_CALL, tid as usize, a, b, c, d).map(|(tid, a, b, c, d)| (tid as u16, a, b, c, d))
}

pub fn itc_reply(a: usize, b: usize, c: usize, d: usize) -> Result<(), Error> {
  syscall_4_0(SYS_ITC_REPLY, a, b, c, d)
}

pub fn server_register(server_id: usize) -> Result<(), Error> {
  syscall_1_0(SYS_SERVER_REGISTER, server_id)
}

pub fn server_tid(server_id: usize) -> Result<u16, Error> {
  syscall_1_1(SYS_SERVER_TID, server_id).map(|x| x as u16)
}

pub fn server_tid_wait(server_id: usize) -> u16 {
  loop {
    if let Ok(tid) = server_tid(server_id) {
      break tid
    }
  }
}
