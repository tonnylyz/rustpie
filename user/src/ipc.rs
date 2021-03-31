use crate::arch::page_table::*;
use crate::config::*;
use crate::syscall::*;

#[repr(C, align(32))]
#[derive(Copy, Clone, Debug)]
pub struct Ipc {
  pub index: u16,
  pub from: u16,
  pub receiving: bool,
  pub value: usize,
  pub address: usize,
  pub attribute: usize,
}

static mut IPC_SELF: usize = 0;

pub fn set_self_ipc(pid: u16) {
  unsafe {
    IPC_SELF = IPC_LIST_BTM + IPC_PCB_SIZE * ((pid - 1) as usize);
  }
}

pub fn get_self_ipc() -> *const Ipc {
  unsafe {
    IPC_SELF as *const Ipc
  }
}

pub fn send(whom: u16, value: usize, src_va: usize, attr: EntryAttribute) {
  loop {
    match ipc_can_send(whom, value, src_va, attr) {
      Ok(_) => { break; }
      //Err(crate::syscall::Error::IpcNotReceivingError) => { process_yield(); }
      Err(e) => { println!("ipc send {:?}", e) }
    }
  }
}

pub fn receive(dst_va: usize) -> (u16, usize, ArchEntryAttribute) {
  ipc_receive(dst_va);
  unsafe {
    ((*get_self_ipc()).from, (*get_self_ipc()).value, ArchEntryAttribute::new((*get_self_ipc()).attribute as u64))
  }
}