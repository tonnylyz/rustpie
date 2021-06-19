use core::mem::size_of;

use SyscallOutRegisters::*;

use crate::arch::{ArchPageTableEntry, PAGE_SIZE};
use crate::lib::address_space::AddressSpace;
use crate::lib::cpu::{CoreTrait, current};
use crate::lib::event::Event::Interrupt;
use crate::lib::interrupt::INTERRUPT_WAIT;
use crate::lib::thread::Status::{TsNotRunnable, TsRunnable, TsWaitForInterrupt};
use crate::lib::traits::*;
use crate::mm::page_table::{Entry, PageTableEntryAttrTrait, PageTableTrait};
use crate::util::round_down;

use self::Error::*;
use core::fmt::{Display, Formatter};
use crate::current_thread;

#[derive(Debug, Copy, Clone)]
pub enum Error {
  InvalidArgumentError = 1,
  OutOfMemoryError,
  MemoryNotMappedError,
  NotReceivingError,
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

impl core::convert::From<crate::mm::page_table::Error> for Error {
  fn from(_: crate::mm::page_table::Error) -> Self {
    InternalError
  }
}

#[derive(Debug)]
pub enum SyscallOutRegisters {
  Unit,
  Single(usize),
  Double(usize, usize),
  Triple(usize, usize, usize),
  Quadruple(usize, usize, usize, usize),
  Pentad(usize, usize, usize, usize, usize),
}

impl Display for SyscallOutRegisters {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    match self {
      Unit => { write!(f, "()")? }
      Single(a) => { write!(f, "({:x})", a)? }
      Double(a, b) => { write!(f, "({:x}, {:x})", a, b)? }
      Triple(a, b, c) => { write!(f, "({:x}, {:x}, {:x})", a, b, c)? }
      Quadruple(a, b, c, d) => { write!(f, "({:x}, {:x}, {:x}, {:x})", a, b, c, d)? }
      Pentad(a, b, c, d, e) => { write!(f, "({:x}, {:x}, {:x}, {:x}, {:x})", a, b, c, d, e)? }
    }
    Ok(())
  }
}

pub type SyscallResult = Result<SyscallOutRegisters, Error>;

static SYSCALL_NAMES: [&str; 21] = [
  "null",
  "putc",
  "get_asid",
  "get_tid",
  "thread_yield",
  "thread_destroy",
  "event_handler",
  "mem_alloc",
  "mem_map",
  "mem_unmap",
  "address_space_alloc",
  "thread_alloc",
  "thread_set_status",
  "ipc_receive",
  "ipc_can_send",
  "itc_receive",
  "itc_send",
  "itc_call",
  "itc_reply",
  "server_register",
  "server_tid",
];

pub trait SyscallTrait {
  fn null() -> SyscallResult;
  fn putc(c: char) -> SyscallResult;
  fn get_asid() -> SyscallResult;
  fn get_tid() -> SyscallResult;
  fn thread_yield() -> SyscallResult;
  fn thread_destroy(asid: u16) -> SyscallResult;
  fn event_handler(asid: u16, value: usize, sp: usize, event: usize) -> SyscallResult;
  fn mem_alloc(asid: u16, va: usize, perm: usize) -> SyscallResult;
  fn mem_map(src_asid: u16, src_va: usize, dst_asid: u16, dst_va: usize, perm: usize) -> SyscallResult;
  fn mem_unmap(asid: u16, va: usize) -> SyscallResult;
  fn address_space_alloc() -> SyscallResult;
  fn thread_alloc(asid: u16, entry: usize, sp: usize, arg: usize) -> SyscallResult;
  fn thread_set_status(pid: u16, status: usize) -> SyscallResult;
  fn ipc_receive(dst_va: usize) -> SyscallResult;
  fn ipc_can_send(pid: u16, value: usize, src_va: usize, perm: usize) -> SyscallResult;
  fn itc_recv() -> SyscallResult;
  fn itc_send(tid: u16, a: usize, b: usize, c: usize, d: usize) -> SyscallResult;
  fn itc_call(tid: u16, a: usize, b: usize, c: usize, d: usize) -> SyscallResult;
  fn itc_reply(a: usize, b: usize, c: usize, d: usize) -> SyscallResult;
  fn server_register(a: usize) -> SyscallResult;
  fn server_tid(a: usize) -> SyscallResult;
}

pub struct Syscall;

fn lookup_as(asid: u16) -> Result<AddressSpace, Error> {
  // TODO: check permission
  match if asid == 0 {
    crate::current_cpu().address_space()
  } else {
    crate::lib::address_space::lookup(asid)
  } {
    None => { Err(InternalError) }
    Some(a) => { Ok(a) }
  }
}

impl SyscallTrait for Syscall {
  fn null() -> SyscallResult {
    Ok(Unit)
  }

  fn putc(c: char) -> SyscallResult {
    crate::driver::uart::putc(c as u8);
    Ok(Unit)
  }

  fn get_asid() -> SyscallResult {
    match crate::current_cpu().address_space() {
      None => { Err(InternalError) }
      Some(a) => { Ok(Single(a.asid() as usize)) }
    }
  }

  fn get_tid() -> SyscallResult {
    match crate::current_cpu().running_thread() {
      None => { Err(InternalError) }
      Some(t) => { Ok(Single(t.tid() as usize)) }
    }
  }

  fn thread_yield() -> SyscallResult {
    crate::current_cpu().schedule();
    Ok(Unit)
  }

  fn thread_destroy(tid: u16) -> SyscallResult {
    let current_thread = crate::current_thread();
    if tid == 0 {
      current_thread.destroy();
      Syscall::thread_yield()
    } else {
      match crate::lib::thread::lookup(tid) {
        None => { Err(PermissionDenied) }
        Some(t) => {
          if t.is_child_of(current_thread.tid()) {
            // TODO: check if destroy safe for inter-processor
            t.destroy();
            Ok(Unit)
          } else {
            Err(PermissionDenied)
          }
        }
      }
    }
  }

  fn event_handler(asid: u16, entry: usize, sp: usize, event: usize) -> SyscallResult {
    // trace!("event_handler {} {:x} {:x} {}", asid, entry, sp, event);
    let e = event.into();
    let a = lookup_as(asid)?;
    a.event_register(e, entry, sp);

    if asid != 0 {
      Ok(Unit)
    } else {
      if let Interrupt(i) = e {
        // register an interrupt event for current thread
        let t = crate::current_thread().clone();
        return match INTERRUPT_WAIT.add_yield(t.clone(), i) {
          Ok(_) => {
            t.set_status(TsWaitForInterrupt);
            Self::thread_yield()
          }
          Err(super::interrupt::Error::AlreadyWaiting) => {
            INTERRUPT_WAIT.remove(i);
            Ok(Unit)
          }
          _ => {
            Err(InternalError)
          }
        };
      } else {
        Ok(Unit)
      }
    }
  }

  fn mem_alloc(asid: u16, va: usize, attr: usize) -> SyscallResult {
    // if va >= CONFIG_USER_LIMIT {
    //   return Err(MemoryLimitError);
    // }
    let p = lookup_as(asid)?;
    let frame = crate::mm::page_pool::try_alloc()?;
    frame.zero();
    let page_table = p.page_table();
    let user_attr = Entry::from(ArchPageTableEntry::from_pte(attr)).attribute();
    let attr = user_attr.filter();
    let uf = crate::mm::UserFrame::new_memory(frame);
    page_table.insert_page(va, uf, attr)?;
    Ok(Unit)
  }

  fn mem_map(src_asid: u16, src_va: usize, dst_asid: u16, dst_va: usize, attr: usize) -> SyscallResult {
    let src_va = round_down(src_va, PAGE_SIZE);
    let dst_va = round_down(dst_va, PAGE_SIZE);
    // if src_va >= CONFIG_USER_LIMIT || dst_va >= CONFIG_USER_LIMIT {
    //   return Err(MemoryLimitError);
    // }
    let src_as = lookup_as(src_asid)?;
    let dst_as = lookup_as(dst_asid)?;
    let src_pt = src_as.page_table();
    let user_attr = Entry::from(ArchPageTableEntry::from_pte(attr)).attribute();
    let attr = user_attr.filter();
    let dst_pt = dst_as.page_table();
    if let Some(uf) = src_pt.lookup_user_page(src_va) {
      dst_pt.insert_page(dst_va, uf, attr)?;
      Ok(Unit)
    } else {
      Err(MemoryNotMappedError)
    }
  }

  fn mem_unmap(asid: u16, va: usize) -> SyscallResult {
    // if va >= CONFIG_USER_LIMIT {
    //   return Err(MemoryLimitError);
    // }
    let a = lookup_as(asid)?;
    let page_table = a.page_table();
    page_table.remove_page(va)?;
    Ok(Unit)
  }

  fn address_space_alloc() -> SyscallResult {
    let a = crate::lib::address_space::alloc();
    Ok(Single(a.asid() as usize))
  }

  fn thread_alloc(asid: u16, entry: usize, sp: usize, arg: usize) -> SyscallResult {
    let a = lookup_as(asid)?;
    let child_thread = crate::lib::thread::new_user(entry, sp, arg, a.clone(), Some(current_thread()));
    Ok(Single(child_thread.tid() as usize))
  }

  fn thread_set_status(tid: u16, status: usize) -> SyscallResult {
    use common::thread::*;
    let status = match status {
      THREAD_STATUS_NOT_RUNNABLE => crate::lib::thread::Status::TsNotRunnable,
      THREAD_STATUS_RUNNABLE => crate::lib::thread::Status::TsRunnable,
      _ => return Err(InvalidArgumentError)
    };
    match crate::lib::thread::lookup(tid) {
      None => {}
      Some(t) => {
        t.set_status(status);
      }
    }
    Ok(Unit)
  }

  #[allow(unused_variables)]
  fn ipc_receive(dst_va: usize) -> SyscallResult {
    todo!()
  }

  #[allow(unused_variables)]
  fn ipc_can_send(pid: u16, value: usize, src_va: usize, attr: usize) -> SyscallResult {
    todo!()
  }

  fn itc_recv() -> SyscallResult {
    let t = current().running_thread().ok_or_else(|| InternalError)?;
    t.clear_peer(); // receive from any sender
    t.sleep();
    crate::current_cpu().schedule();
    Ok(Unit)
  }

  fn itc_send(tid: u16, a: usize, b: usize, c: usize, d: usize) -> SyscallResult {
    let t = current().running_thread().ok_or_else(|| InternalError)?;
    let target = crate::lib::thread::lookup(tid).ok_or_else(|| InvalidArgumentError)?;
    if t.address_space() != target.address_space() {
      return Err(PermissionDenied);
    }
    if !target.receivable(&t) {
      return Err(InternalError);
    }
    let mut ctx = target.context();
    ctx.set_syscall_result(&SyscallResult::Ok(SyscallOutRegisters::Pentad(t.tid() as usize, a, b, c, d)));
    target.set_context(ctx);
    target.wake();
    Ok(Unit)
  }

  fn itc_call(tid: u16, a: usize, b: usize, c: usize, d: usize) -> SyscallResult {
    let t = current().running_thread().ok_or_else(|| InternalError)?;
    let target = crate::lib::thread::lookup(tid).ok_or_else(|| InvalidArgumentError)?;
    if t.address_space() != target.address_space() {
      return Err(PermissionDenied);
    }
    if !target.receivable(&t) {
      return Err(InternalError);
    }
    let mut ctx = target.context();
    ctx.set_syscall_result(&SyscallResult::Ok(SyscallOutRegisters::Pentad(t.tid() as usize, a, b, c, d)));
    target.set_context(ctx);

    target.wake();
    target.set_peer(t.clone());
    t.set_peer(target.clone());
    t.sleep();

    crate::current_cpu().schedule();
    Ok(Unit)
  }

  fn itc_reply(a: usize, b: usize, c: usize, d: usize) -> SyscallResult {
    let t = current().running_thread().ok_or(InternalError)?;
    let target = t.peer().ok_or(InternalError)?;
    if t.address_space() != target.address_space() {
      return Err(PermissionDenied);
    }
    if !target.receivable(&t) {
      return Err(InternalError);
    }
    let mut ctx = target.context();
    ctx.set_syscall_result(&SyscallResult::Ok(SyscallOutRegisters::Pentad(t.tid() as usize, a, b, c, d)));
    target.set_context(ctx);

    target.clear_peer();
    t.clear_peer();
    target.wake();
    Ok(Unit)
  }

  fn server_register(server_id: usize) -> SyscallResult {
    use common::server::*;
    let t = current().running_thread().ok_or(InternalError)?;
    super::server::set(server_id, t.tid());
    Ok(Unit)
  }

  fn server_tid(server_id: usize) -> SyscallResult {
    match super::server::get(server_id) {
      None => {
        Err(InvalidArgumentError)
      },
      Some(tid) => {
        Ok(Single(tid as usize))
      }
    }
  }
}

pub fn syscall() {
  use common::syscall::*;
  let ctx = crate::current_cpu().context_mut();
  let tid = crate::current_thread().tid();
  let arg = |i: usize| { ctx.syscall_argument(i) };
  let num = ctx.syscall_number();
  let result = match num {
    SYS_NULL => Syscall::null(),
    SYS_PUTC => Syscall::putc(arg(0) as u8 as char),
    SYS_GET_ASID => Syscall::get_asid(),
    SYS_GET_TID => Syscall::get_tid(),
    SYS_THREAD_YIELD => Syscall::thread_yield(),
    SYS_THREAD_DESTROY => Syscall::thread_destroy(arg(0) as u16),
    SYS_EVENT_HANDLER => Syscall::event_handler(arg(0) as u16, arg(1), arg(2), arg(3)),
    SYS_MEM_ALLOC => Syscall::mem_alloc(arg(0) as u16, arg(1), arg(2)),
    SYS_MEM_MAP => Syscall::mem_map(arg(0) as u16, arg(1), arg(2) as u16, arg(3), arg(4)),
    SYS_MEM_UNMAP => Syscall::mem_unmap(arg(0) as u16, arg(1)),
    SYS_ADDRESS_SPACE_ALLOC => Syscall::address_space_alloc(),
    SYS_THREAD_ALLOC => Syscall::thread_alloc(arg(0) as u16, arg(1), arg(2), arg(3)),
    SYS_THREAD_SET_STATUS => Syscall::thread_set_status(arg(0) as u16, arg(1)),
    SYS_IPC_RECEIVE => Syscall::ipc_receive(arg(0)),
    SYS_CAN_SEND => Syscall::ipc_can_send(arg(0) as u16, arg(1), arg(2), arg(3)),
    SYS_ITC_RECV => Syscall::itc_recv(),
    SYS_ITC_SEND => Syscall::itc_send(arg(0) as u16, arg(1), arg(2), arg(3), arg(4)),
    SYS_ITC_CALL => Syscall::itc_call(arg(0) as u16, arg(1), arg(2), arg(3), arg(4)),
    SYS_ITC_REPLY => Syscall::itc_reply(arg(1), arg(2), arg(3), arg(4)),
    SYS_SERVER_REGISTER => Syscall::server_register(arg(0)),
    SYS_SERVER_TID => Syscall::server_tid(arg(0)),
    _ => {
      warn!("system call: unrecognized system call number");
      Err(super::syscall::Error::InvalidArgumentError)
    }
  };

  match result {
    Ok(ref regs) => {
      if num != 1 {
        trace!("#{} {} t{} Ok {}", num, SYSCALL_NAMES[num], tid, regs);
      }
    }
    Err(err) => {
      trace!("#{} {} t{} Err {:x?}", num, SYSCALL_NAMES[num], tid, err);
    }
  }
  ctx.set_syscall_result(&result);
}