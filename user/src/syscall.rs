use crate::arch::page_table::*;

pub type Error = (); 

#[allow(dead_code)]
pub enum ThreadStatus {
  TsRunnable = 1,
  TsNotRunnable = 2,
}

extern "C" {
  fn syscall_1(x0: u8);
  fn syscall_2(tid: u16) -> u16;
  fn syscall_3();
  fn syscall_4(asid: u16) -> isize;
  fn syscall_5(asid: u16, value: usize, sp: usize, event: usize) -> isize;
  fn syscall_6(asid: u16, va: usize, attr: usize) -> isize;
  fn syscall_7(src_asid: u16, src_va: usize, dst_asid: u16, dst_va: usize, attr: usize) -> isize;
  fn syscall_8(asid: u16, va: usize) -> isize;
  fn syscall_9() -> isize;
  fn syscall_10(tid: u16, status: usize) -> isize;
  fn syscall_11(dst_va: usize);
  fn syscall_12(asid: u16, value: usize, src_va: usize, attr: usize) -> isize;
  fn syscall_13(entry: usize, sp: usize, arg: usize) -> isize;
}

pub fn putc(c: char) {
  unsafe { syscall_1(c as u8); }
}

pub fn get_asid() -> u16 {
  unsafe { syscall_2(0) }
}

pub fn thread_yield() {
  unsafe { syscall_3(); }
}

pub fn thread_destroy(tid: u16) -> Result<(), Error> {
  match unsafe { syscall_4(tid) } {
    -1 => Err(()),
    _ => Ok(()),
  }
}

pub fn event_handler(asid: u16, value: usize, sp: usize, event: usize) -> Result<(), Error> {
  match unsafe { syscall_5(asid, value, sp, event) } {
    -1 => Err(()),
    _ => Ok(()),
  }
}

pub fn mem_alloc(asid: u16, va: usize, attr: EntryAttribute) -> Result<(), Error> {
  let attr = ArchEntryAttribute::from(attr).to_usize();
  match unsafe { syscall_6(asid, va, attr) } {
    -1 => Err(()),
    _ => Ok(()),
  }
}

pub fn mem_map(src_asid: u16, src_va: usize, dst_asid: u16, dst_va: usize, attr: EntryAttribute) -> Result<(), Error> {
  let attr = ArchEntryAttribute::from(attr).to_usize();
  match unsafe { syscall_7(src_asid, src_va, dst_asid, dst_va, attr) } {
    -1 => Err(()),
    _ => Ok(()),
  }
}

pub fn mem_unmap(asid: u16, va: usize) -> Result<(), Error> {
  match unsafe { syscall_8(asid, va) } {
    -1 => Err(()),
    _ => Ok(()),
  }
}

#[inline(always)]
pub fn address_space_alloc() -> Result<(u16, u16), Error> {
  match unsafe { syscall_9() } {
    -1 => Err(()),
    x => Ok(((x >> 16) as u16, x as u16)),
  }
}

pub fn thread_set_status(tid: u16, status: ThreadStatus) -> Result<(), Error> {
  match unsafe { syscall_10(tid, status as usize) } {
    -1 => Err(()),
    _ => Ok(()),
  }
}

pub fn ipc_receive(dst_va: usize) {
  unsafe { syscall_11(dst_va); }
}

pub fn ipc_can_send(asid: u16, value: usize, src_va: usize, attr: EntryAttribute) -> Result<(), Error> {
  let attr = ArchEntryAttribute::from(attr).to_usize();
  match unsafe { syscall_12(asid, value, src_va, attr) } {
    -1 => Err(()),
    _ => Ok(()),
  }
}

pub fn thread_alloc(entry: usize, sp: usize, arg: usize) -> Result<u16, Error> {
  match unsafe { syscall_13(entry, sp, arg) } {
    -1 => Err(()),
    x => { Ok(x as u16) }
  }
}