use crate::arch::page_table::*;

pub type Error = ();

#[allow(dead_code)]
pub enum ThreadStatus {
  TsRunnable = 1,
  TsNotRunnable = 2,
}

extern "C" {
  fn syscall_0(); // null
fn syscall_1(x0: u8); // putc
fn syscall_2(tid: u16) -> u16; // get_asid
fn syscall_3() -> u16; // get_tid
fn syscall_4(); // thread_yield
fn syscall_5(asid: u16) -> isize; // thread_destroy
fn syscall_6(asid: u16, value: usize, sp: usize, event: usize) -> isize; // event_handler
fn syscall_7(asid: u16, va: usize, attr: usize) -> isize; // mem_alloc
fn syscall_8(src_asid: u16, src_va: usize, dst_asid: u16, dst_va: usize, attr: usize) -> isize; // mem_map
fn syscall_9(asid: u16, va: usize) -> isize; // mem_unmap
fn syscall_10() -> isize; // address_space_alloc
fn syscall_11(entry: usize, sp: usize, arg: usize) -> isize; // thread_alloc
fn syscall_12(tid: u16, status: usize) -> isize; // thread_set_status
fn syscall_13(dst_va: usize); // ipc_receive
fn syscall_14(asid: u16, value: usize, src_va: usize, attr: usize) -> isize; // ipc_can_send
fn syscall_15(msg_ptr: usize);
  fn syscall_16(tid: u16, a: usize, b: usize, c: usize, d: usize) -> isize;
}

pub fn null() { unsafe { syscall_0(); } }

pub fn putc(c: char) {
  unsafe { syscall_1(c as u8); }
}

pub fn get_asid(tid: u16) -> u16 {
  unsafe { syscall_2(tid) }
}

pub fn get_tid() -> u16 { unsafe { syscall_3() } }

pub fn thread_yield() {
  unsafe { syscall_4(); }
}

pub fn thread_destroy(tid: u16) -> Result<(), Error> {
  match unsafe { syscall_5(tid) } {
    -1 => Err(()),
    _ => Ok(()),
  }
}

pub fn event_handler(asid: u16, value: usize, sp: usize, event: usize) -> Result<(), Error> {
  match unsafe { syscall_6(asid, value, sp, event) } {
    -1 => Err(()),
    _ => Ok(()),
  }
}

pub fn mem_alloc(asid: u16, va: usize, attr: EntryAttribute) -> Result<(), Error> {
  let attr = ArchEntryAttribute::from(attr).to_usize();
  match unsafe { syscall_7(asid, va, attr) } {
    -1 => Err(()),
    _ => Ok(()),
  }
}

pub fn mem_map(src_asid: u16, src_va: usize, dst_asid: u16, dst_va: usize, attr: EntryAttribute) -> Result<(), Error> {
  let attr = ArchEntryAttribute::from(attr).to_usize();
  match unsafe { syscall_8(src_asid, src_va, dst_asid, dst_va, attr) } {
    -1 => Err(()),
    _ => Ok(()),
  }
}

pub fn mem_unmap(asid: u16, va: usize) -> Result<(), Error> {
  match unsafe { syscall_9(asid, va) } {
    -1 => Err(()),
    _ => Ok(()),
  }
}

#[inline(always)]
pub fn address_space_alloc() -> Result<(u16, u16), Error> {
  match unsafe { syscall_10() } {
    -1 => Err(()),
    x => Ok(((x >> 16) as u16, x as u16)),
  }
}

pub fn thread_alloc(entry: usize, sp: usize, arg: usize) -> Result<u16, Error> {
  match unsafe { syscall_11(entry, sp, arg) } {
    -1 => Err(()),
    x => { Ok(x as u16) }
  }
}

pub fn thread_set_status(tid: u16, status: ThreadStatus) -> Result<(), Error> {
  match unsafe { syscall_12(tid, status as usize) } {
    -1 => Err(()),
    _ => Ok(()),
  }
}

pub fn ipc_receive(dst_va: usize) {
  unsafe { syscall_13(dst_va); }
}

pub fn ipc_can_send(asid: u16, value: usize, src_va: usize, attr: EntryAttribute) -> Result<(), Error> {
  let attr = ArchEntryAttribute::from(attr).to_usize();
  match unsafe { syscall_14(asid, value, src_va, attr) } {
    -1 => Err(()),
    _ => Ok(()),
  }
}

pub fn itc_receive(msg_ptr: usize) {
  unsafe {
    syscall_15(msg_ptr);
  }
}

pub fn itc_send(tid: u16, a: usize, b: usize, c: usize, d: usize) -> isize {
  unsafe {
    syscall_16(tid, a, b, c, d)
  }
}