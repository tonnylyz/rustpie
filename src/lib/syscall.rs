use crate::arch::{ArchPageTableEntry, ArchPageTableEntryTrait, ContextFrameTrait, PAGE_SIZE};
use crate::config::CONFIG_USER_LIMIT;
use crate::lib::{round_down};
use crate::lib::address_space::{AddressSpace};
use crate::lib::core::{CoreTrait, current};
use crate::lib::page_table::{Entry, PageTableEntryAttrTrait, PageTableTrait};

use self::Error::*;

#[derive(Debug)]
pub enum Error {
  InvalidArgumentError = 1,
  _OutOfProcessError,
  OutOfMemoryError,
  MemoryLimitError,
  MemoryNotMappedError,
  _IpcNotReceivingError,
  InternalError,
  PermissionDenied,
}

impl core::convert::From<crate::mm::page_pool::Error> for Error {
  fn from(e: crate::mm::page_pool::Error) -> Self {
    match e {
      crate::mm::page_pool::Error::OutOfFrame => { OutOfMemoryError }
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

impl Into<SyscallResult> for SystemCallValue {
  fn into(self) -> SyscallResult {
    Ok(self)
  }
}

#[derive(Debug)]
pub enum SystemCallValue {
  Unit,
  U32(u32),
  U16(u16),
  ISize(isize),
  USize(usize),
}
use SystemCallValue::*;
use crate::lib::event::Event;
use crate::lib::thread::Thread;

pub type SyscallResult = Result<SystemCallValue, Error>;

pub trait SystemCallTrait {
  fn null() -> SyscallResult;
  fn putc(c: char) -> SyscallResult;
  fn get_asid(tid: u16) -> SyscallResult;
  fn get_tid() -> SyscallResult;
  fn thread_yield() -> SyscallResult;
  fn thread_destroy(asid: u16) -> SyscallResult;
  fn event_handler(asid: u16, value: usize, sp: usize, event: usize) -> SyscallResult;
  fn mem_alloc(asid: u16, va: usize, perm: usize) -> SyscallResult;
  fn mem_map(src_asid: u16, src_va: usize, dst_asid: u16, dst_va: usize, perm: usize) -> SyscallResult;
  fn mem_unmap(asid: u16, va: usize) -> SyscallResult;
  fn address_space_alloc() -> SyscallResult;
  fn thread_alloc(entry: usize, sp: usize, arg: usize) -> SyscallResult;
  fn thread_set_status(pid: u16, status: crate::lib::thread::Status) -> SyscallResult;
  fn ipc_receive(dst_va: usize) -> SyscallResult;
  fn ipc_can_send(pid: u16, value: usize, src_va: usize, perm: usize) -> SyscallResult;
  fn itc_receive(msg_ptr: usize) -> SyscallResult;
  fn itc_can_send(tid: u16, a: usize, b: usize, c: usize, d: usize) -> SyscallResult;
}

pub struct SystemCall;

fn lookup_as(asid: u16) -> Result<AddressSpace, Error> {
  // TODO: check permission
  match if asid == 0 {
    crate::lib::core::current().address_space()
  } else {
    crate::lib::address_space::lookup(asid)
  } {
    None => { Err(InternalError) }
    Some(a) => { Ok(a) }
  }
}

const OK: SyscallResult = Ok(SystemCallValue::Unit);

impl SystemCallTrait for SystemCall {
  fn null() -> SyscallResult {
    OK
  }

  fn putc(c: char) -> SyscallResult {
    crate::driver::uart::putc(c as u8);
    OK
  }

  fn get_asid(_tid: u16) -> SyscallResult {
    match crate::lib::core::current().address_space() {
      None => { Err(InternalError) }
      Some(a) => { U16(a.asid()).into() }
    }
  }

  fn get_tid() -> SyscallResult {
    match crate::lib::core::current().running_thread() {
      None => { Err(InternalError) }
      Some(t) => { U16(t.tid()).into() }
    }
  }

  fn thread_yield() -> SyscallResult {
    crate::lib::core::current().schedule();
    OK
  }

  fn thread_destroy(tid: u16) -> SyscallResult {
    let current_thread = current().running_thread().unwrap();
    if tid == 0 {
      println!("current thread {} destroyed", current_thread.tid());
      current_thread.destroy();
      SystemCall::thread_yield()
    } else {
      let target = crate::lib::thread::lookup(tid);
      match target {
        None => { Err(PermissionDenied) }
        Some(t) => {
          if t.is_child_of(current_thread.tid()) {
            // TODO: check if destroy safe for inter-processor
            t.destroy();
            OK
          } else {
            Err(PermissionDenied)
          }
        }
      }
    }
  }

  fn event_handler(asid: u16, entry: usize, sp: usize, event: usize) -> SyscallResult {
    let e = match event {
      0 => Event::PageFault,
      x => Event::Interrupt(x),
    };
    let a = lookup_as(asid)?;
    a.event_register(e, entry, sp);

    if asid == 0 && event != 0 {
      // register an interrupt event for current thread
      crate::lib::interrupt::INTERRUPT_WAIT.add(current().running_thread().unwrap().clone(), event);
    }
    OK
  }

  fn mem_alloc(asid: u16, va: usize, attr: usize) -> SyscallResult {
    if va >= CONFIG_USER_LIMIT {
      return Err(MemoryLimitError);
    }
    let p = lookup_as(asid)?;
    let frame = crate::mm::page_pool::try_alloc()?;
    frame.zero();
    let page_table = p.page_table();
    let user_attr = Entry::from(ArchPageTableEntry::from_pte(attr)).attribute();
    let attr = user_attr.filter();
    let uf = crate::mm::UserFrame::new_memory(frame);
    page_table.insert_page(va, uf, attr)?;
    OK
  }

  fn mem_map(src_asid: u16, src_va: usize, dst_asid: u16, dst_va: usize, attr: usize) -> SyscallResult {
    let src_va = round_down(src_va, PAGE_SIZE);
    let dst_va = round_down(dst_va, PAGE_SIZE);
    if src_va >= CONFIG_USER_LIMIT || dst_va >= CONFIG_USER_LIMIT {
      return Err(MemoryLimitError);
    }
    let src_as = lookup_as(src_asid)?;
    let dst_as = lookup_as(dst_asid)?;
    let src_pt = src_as.page_table();
    let user_attr = Entry::from(ArchPageTableEntry::from_pte(attr)).attribute();
    let attr = user_attr.filter();
    let dst_pt = dst_as.page_table();
    if let Some(uf) = src_pt.lookup_user_page(src_va) {
      dst_pt.insert_page(dst_va, uf, attr)?;
      OK
    } else {
      Err(MemoryNotMappedError)
    }
  }

  fn mem_unmap(asid: u16, va: usize) -> SyscallResult {
    if va >= CONFIG_USER_LIMIT {
      return Err(MemoryLimitError);
    }
    let a = lookup_as(asid)?;
    let page_table = a.page_table();
    page_table.remove_page(va)?;
    OK
  }

  fn address_space_alloc() -> SyscallResult {
    let a = crate::lib::address_space::alloc();
    let mut ctx = *crate::lib::core::current().context();
    ctx.set_syscall_return_value(0);
    let t = crate::lib::thread::new_user(0, 0, 0, a.clone(), current().running_thread());
    t.set_context(ctx);
    t.set_status(crate::lib::thread::Status::TsNotRunnable);
    U32(((a.asid() as u32) << 16) | (t.tid() as u32)).into()
  }

  fn thread_alloc(entry: usize, sp: usize, arg: usize) -> SyscallResult {
    let t = crate::lib::core::current().running_thread().unwrap();
    let p = t.address_space().unwrap();
    let child_thread = crate::lib::thread::new_user(entry, sp, arg, p.clone(), current().running_thread());
    child_thread.set_status(crate::lib::thread::Status::TsRunnable);
    U16(child_thread.tid()).into()
  }

  fn thread_set_status(tid: u16, status: crate::lib::thread::Status) -> SyscallResult {
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
    OK
  }

  #[allow(unused_variables)]
  fn ipc_receive(dst_va: usize)-> SyscallResult {
    unimplemented!()
  }

  #[allow(unused_variables)]
  fn ipc_can_send(pid: u16, value: usize, src_va: usize, attr: usize) -> SyscallResult {
    unimplemented!()
  }

  #[allow(unused_variables)]
  fn itc_receive(msg_ptr: usize)-> SyscallResult {
    unimplemented!()
  }

  #[allow(unused_variables)]
  fn itc_can_send(tid: u16, a: usize, b: usize, c: usize, d: usize) -> SyscallResult {
    unimplemented!()
  }
}