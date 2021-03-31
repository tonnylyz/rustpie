use crate::arch::{ArchPageTableEntry, ArchPageTableEntryTrait, ContextFrameTrait, PAGE_SIZE};
use crate::config::CONFIG_USER_LIMIT;
use crate::lib::{round_down};
use crate::lib::address_space::{AddressSpace, Asid};
use crate::lib::core::{CoreTrait, current};
use crate::lib::page_table::{Entry, PageTableEntryAttrTrait, PageTableTrait};
use crate::lib::thread::Tid;

use self::Error::*;

pub enum Error {
  InvalidArgumentError = 1,
  _OutOfProcessError,
  OutOfMemoryError,
  MemoryLimitError,
  MemoryNotMappedError,
  _IpcNotReceivingError,
  InternalError,
}

impl core::convert::From<crate::mm::page_pool::Error> for Error {
  fn from(e: crate::mm::page_pool::Error) -> Self {
    match e {
      crate::mm::page_pool::Error::OutOfFrameError => { OutOfMemoryError }
      _ => { InternalError }
    }
  }
}

impl core::convert::From<crate::lib::page_table::Error> for Error {
  fn from(_: crate::lib::page_table::Error) -> Self {
    InternalError
  }
}

impl core::convert::From<crate::lib::address_space::Error> for Error {
  fn from(e: crate::lib::address_space::Error) -> Self {
    match e {
      _ => { InternalError }
    }
  }
}

pub trait SystemCallTrait {
  fn putc(c: char);
  fn get_asid() -> u16;
  fn get_tid() -> u16;
  fn thread_yield();
  fn address_space_destroy(pid: u16) -> Result<(), Error>;
  fn process_set_exception_handler(pid: u16, value: usize, sp: usize) -> Result<(), Error>;
  fn mem_alloc(pid: u16, va: usize, perm: usize) -> Result<(), Error>;
  fn mem_map(src_pid: u16, src_va: usize, dst_pid: u16, dst_va: usize, perm: usize) -> Result<(), Error>;
  fn mem_unmap(pid: u16, va: usize) -> Result<(), Error>;
  fn address_space_alloc() -> Result<u16, Error>;
  fn thread_alloc(entry: usize, sp: usize, arg: usize) -> Result<u16, Error>;
  fn thread_set_status(pid: u16, status: crate::lib::thread::Status) -> Result<(), Error>;
  fn ipc_receive(dst_va: usize);
  fn ipc_can_send(pid: u16, value: usize, src_va: usize, perm: usize) -> Result<(), Error>;
  fn itc_receive(msg_ptr: usize);
  fn itc_can_send(tid: u16, a: usize, b: usize, c: usize, d: usize) -> Result<(), Error>;
}

pub struct SystemCall;

fn lookup_as(asid: u16) -> Result<AddressSpace, Error> {
  match if asid == 0 {
    crate::lib::core::current().address_space()
  } else {
    crate::lib::address_space::lookup(asid)
  } {
    None => { Err(InternalError) }
    Some(a) => { Ok(a) }
  }
}

impl SystemCallTrait for SystemCall {
  fn putc(c: char) {
    crate::driver::uart::putc(c as u8);
  }

  fn get_asid() -> u16 {
    match crate::lib::core::current().address_space() {
      None => { 0 }
      Some(a) => { a.asid() }
    }
  }

  fn get_tid() -> u16 {
    match crate::lib::core::current().running_thread() {
      None => { 0 }
      Some(t) => { t.tid() }
    }
  }

  fn thread_yield() {
    crate::lib::core::current().schedule();
  }

  fn address_space_destroy(asid: u16) -> Result<(), Error> {
    let p = lookup_as(asid)?;
    p.destroy();
    Ok(())
  }

  #[allow(unused_variables)]
  fn process_set_exception_handler(pid: u16, entry: usize, stack_top: usize) -> Result<(), Error> {
    unimplemented!()
  }

  fn mem_alloc(asid: u16, va: usize, attr: usize) -> Result<(), Error> {
    if va >= CONFIG_USER_LIMIT {
      return Err(MemoryLimitError);
    }
    let p = lookup_as(asid)?;
    let frame = crate::mm::page_pool::try_alloc()?;
    frame.zero();
    let page_table = p.page_table();
    let user_attr = Entry::from(ArchPageTableEntry::from_pte(attr)).attribute();
    let attr = user_attr.filter();
    page_table.insert_page(va, frame, attr)?;
    Ok(())
  }

  fn mem_map(src_asid: u16, src_va: usize, dst_asid: u16, dst_va: usize, attr: usize) -> Result<(), Error> {
    let src_va = round_down(src_va, PAGE_SIZE);
    let dst_va = round_down(dst_va, PAGE_SIZE);
    if src_va >= CONFIG_USER_LIMIT || dst_va >= CONFIG_USER_LIMIT {
      return Err(MemoryLimitError);
    }
    let src_as = lookup_as(src_asid)?;
    let dst_as = lookup_as(dst_asid)?;
    let src_pt = src_as.page_table();
    if let Some(pte) = src_pt.lookup_page(src_va) {
      let pa = pte.pa();
      let user_attr = Entry::from(ArchPageTableEntry::from_pte(attr)).attribute();
      let attr = user_attr.filter();
      let dst_pt = dst_as.page_table();
      dst_pt.insert_page(dst_va, crate::mm::PageFrame::new(pa), attr)?;
      Ok(())
    } else {
      Err(MemoryNotMappedError)
    }
  }

  fn mem_unmap(asid: u16, va: usize) -> Result<(), Error> {
    if va >= CONFIG_USER_LIMIT {
      return Err(MemoryLimitError);
    }
    let a = lookup_as(asid)?;
    let page_table = a.page_table();
    page_table.remove_page(va)?;
    Ok(())
  }

  fn address_space_alloc() -> Result<Asid, Error> {
    let new_as = crate::lib::address_space::alloc();
    let mut ctx = *crate::lib::core::current().context();
    ctx.set_syscall_return_value(0);
    let child_thread = crate::lib::thread::new_user(0, 0, 0, new_as.clone(), current().running_thread());
    child_thread.set_context(ctx);
    child_thread.set_status(crate::lib::thread::Status::TsNotRunnable);
    Ok(new_as.asid())
  }

  fn thread_alloc(entry: usize, sp: usize, arg: usize) -> Result<Tid, Error> {
    let t = crate::lib::core::current().running_thread().unwrap();
    let p = t.address_space().unwrap();
    let child_thread = crate::lib::thread::new_user(entry, sp, arg, p.clone(), current().running_thread());
    child_thread.set_status(crate::lib::thread::Status::TsRunnable);
    Ok(child_thread.tid())
  }

  fn thread_set_status(tid: u16, status: crate::lib::thread::Status) -> Result<(), Error> {
    use crate::lib::thread::Status::{TsRunnable, TsNotRunnable};
    if status != TsRunnable && status != TsNotRunnable {
      return Err(InvalidArgumentError);
    }
    match crate::lib::thread::lookup(tid) {
      None => {}
      Some(t) => {
        t.set_status(status);
      }
    }
    Ok(())
  }

  #[allow(unused_variables)]
  fn ipc_receive(dst_va: usize) {
    unimplemented!()
  }

  #[allow(unused_variables)]
  fn ipc_can_send(pid: u16, value: usize, src_va: usize, attr: usize) -> Result<(), Error> {
    unimplemented!()
  }

  #[allow(unused_variables)]
  fn itc_receive(msg_ptr: usize) {
    unimplemented!()
  }

  #[allow(unused_variables)]
  fn itc_can_send(tid: u16, a: usize, b: usize, c: usize, d: usize) -> Result<(), Error> {
    unimplemented!()
  }
}