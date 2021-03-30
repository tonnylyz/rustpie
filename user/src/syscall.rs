use Error::*;

use crate::arch::page_table::*;

#[derive(Debug)]
pub enum Error {
  InvalidArgumentError = 1,
  OutOfProcessError,
  OutOfMemoryError,
  ProcessPidNotFoundError,
  ProcessParentNotFoundError,
  ProcessParentMismatchedError,
  MemoryLimitError,
  MemoryNotMappedError,
  IpcNotReceivingError,
  InternalError,
}

#[allow(dead_code)]
pub enum ProcessStatus {
  PsRunnable = 1,
  PsNotRunnable = 2,
}

extern "C" {
  fn syscall_1(x0: u8);
  fn syscall_2() -> u16;
  fn syscall_3();
  fn syscall_4(pid: u16) -> isize;
  fn syscall_5(pid: u16, value: usize, sp: usize) -> isize;
  fn syscall_6(pid: u16, va: usize, attr: usize) -> isize;
  fn syscall_7(src_pid: u16, src_va: usize, dst_pid: u16, dst_va: usize, attr: usize) -> isize;
  fn syscall_8(pid: u16, va: usize) -> isize;
  fn syscall_9() -> isize;
  fn syscall_10(pid: u16, status: usize) -> isize;
  fn syscall_11(dst_va: usize);
  fn syscall_12(pid: u16, value: usize, src_va: usize, attr: usize) -> isize;
  fn syscall_13(entry: usize, sp: usize, arg: usize) -> isize;
}

trait SystemCallResult {
  fn to_result(&self) -> Result<(), Error>;
  fn to_result_with<T>(&self) -> Result<T, Error> where T: core::convert::TryFrom<isize>;
}

fn i2err<T>(i: isize) -> Result<T, Error> {
  Err(match i {
    -1 => InvalidArgumentError,
    -2 => OutOfProcessError,
    -3 => OutOfMemoryError,
    -4 => ProcessPidNotFoundError,
    -5 => ProcessParentNotFoundError,
    -6 => ProcessParentMismatchedError,
    -7 => MemoryLimitError,
    -8 => MemoryNotMappedError,
    -9 => IpcNotReceivingError,
    -10 => InternalError,
    _ => InternalError
  })
}

impl SystemCallResult for isize {
  fn to_result(&self) -> Result<(), Error> {
    let i = self.clone();
    match i {
      0 => Ok(()),
      _ => i2err(i)
    }
  }

  fn to_result_with<T>(&self) -> Result<T, Error> where T: core::convert::TryFrom<isize> {
    let i = self.clone();
    if i >= 0 {
      match T::try_from(i) {
        Ok(r) => { Ok(r) }
        Err(_) => { Err(InternalError) }
      }
    } else {
      i2err(i)
    }
  }
}

pub fn putc(c: char) {
  unsafe { syscall_1(c as u8); }
}

pub fn getpid() -> u16 {
  unsafe { syscall_2() }
}

pub fn process_yield() {
  unsafe { syscall_3() }
}

pub fn process_destroy(pid: u16) -> Result<(), Error> {
  unsafe { syscall_4(pid).to_result() }
}

pub fn process_set_exception_handler(pid: u16, value: usize, sp: usize) -> Result<(), Error> {
  unsafe { syscall_5(pid, value, sp).to_result() }
}

pub fn mem_alloc(pid: u16, va: usize, attr: EntryAttribute) -> Result<(), Error> {
  let attr = ArchEntryAttribute::from(attr).to_usize();
  unsafe { syscall_6(pid, va, attr).to_result() }
}

pub fn mem_map(src_pid: u16, src_va: usize, dst_pid: u16, dst_va: usize, attr: EntryAttribute) -> Result<(), Error> {
  let attr = ArchEntryAttribute::from(attr).to_usize();
  unsafe { syscall_7(src_pid, src_va, dst_pid, dst_va, attr).to_result() }
}

pub fn mem_unmap(pid: u16, va: usize) -> Result<(), Error> {
  unsafe { syscall_8(pid, va).to_result() }
}

#[inline(always)]
pub fn process_alloc() -> Result<u16, Error> {
  unsafe { syscall_9().to_result_with::<u16>() }
}

pub fn process_set_status(pid: u16, status: ProcessStatus) -> Result<(), Error> {
  unsafe { syscall_10(pid, status as usize).to_result() }
}

pub fn ipc_receive(dst_va: usize) {
  unsafe { syscall_11(dst_va); }
}

pub fn ipc_can_send(pid: u16, value: usize, src_va: usize, attr: EntryAttribute) -> Result<(), Error> {
  let attr = ArchEntryAttribute::from(attr).to_usize();
  unsafe { syscall_12(pid, value, src_va, attr).to_result() }
}

pub fn thread_alloc(entry: usize, sp: usize, arg: usize) -> Result<u16, Error> {
  unsafe {
    syscall_13(entry, sp, arg).to_result_with::<u16>()
  }
}