use core::fmt::{Display, Formatter};

use common::syscall::error::*;

use SyscallOutRegisters::*;

use crate::arch::{ArchPageTableEntry, PAGE_SIZE};
use crate::lib::address_space::{AddressSpace, address_space_destroy};
use crate::lib::cpu::cpu;
use crate::lib::event::{Event, thread_exit_sem};
use crate::lib::interrupt::INT_SEM;
use crate::lib::semaphore::SemaphoreWaitResult;
use crate::lib::traits::*;
use crate::mm::page_table::{Entry, PageTableEntryAttrTrait, PageTableTrait};
use crate::util::round_down;
use crate::lib::thread::{Tid, thread_destroy, thread_wake, thread_sleep, Thread};

pub type Error = usize;

#[derive(Debug)]
#[allow(dead_code)]
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
  "event_wait",
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

pub struct Syscall;

fn lookup_as(asid: u16) -> Result<AddressSpace, Error> {
  // TODO: check permission
  let a = if asid == 0 {
    current_thread()?.address_space()
  } else {
    crate::lib::address_space::address_space_lookup(asid)
  };
  a.ok_or(ERROR_INVARG)
}

fn current_thread() -> Result<Thread, Error> {
  match cpu().running_thread() {
    None => Err(ERROR_INTERNAL),
    Some(t) => Ok(t),
  }
}

impl Syscall {
  fn null() -> SyscallResult {
    Ok(Unit)
  }

  fn putc(c: char) -> SyscallResult {
    crate::driver::uart::putc(c as u8);
    Ok(Unit)
  }

  fn get_asid(tid: Tid) -> SyscallResult {
    if tid == 0 {
      match crate::lib::cpu::cpu().address_space() {
        None => Err(ERROR_INTERNAL),
        Some(a) => Ok(Single(a.asid() as usize)),
      }
    } else {
      match crate::lib::thread::thread_lookup(tid) {
        None => Err(ERROR_INVARG),
        Some(t) => match t.address_space() {
          None => Err(ERROR_INVARG),
          Some(a) => Ok(Single(a.asid() as usize)),
        },
      }
    }
  }

  fn get_tid() -> SyscallResult {
    Ok(Single(current_thread()?.tid()))
  }

  fn thread_yield() -> SyscallResult {
    crate::lib::cpu::cpu().schedule();
    Ok(Unit)
  }

  fn thread_destroy(tid: Tid) -> SyscallResult {
    let current_thread = current_thread()?;
    if tid == 0 {
      thread_destroy(current_thread);
      Syscall::thread_yield()
    } else {
      match crate::lib::thread::thread_lookup(tid) {
        None => Err(ERROR_DENIED),
        Some(t) => {
          if t.is_child_of(current_thread.tid()) {
            // TODO: check if destroy safe for inter-processor
            thread_destroy(t);
            Ok(Unit)
          } else {
            Err(ERROR_DENIED)
          }
        }
      }
    }
  }

  fn event_wait(event_type: usize, event_num: usize) -> SyscallResult {
    let t = current_thread()?;
    if let Some(e) = Event::from(event_type, event_num) {
      match e {
        Event::Interrupt(i) => {
          match INT_SEM.wait(t.clone(), i) {
            SemaphoreWaitResult::Acquired => Ok(Unit),
            SemaphoreWaitResult::Enqueued => Self::thread_yield(),
          }
        }
        Event::ThreadExit => {
          thread_exit_sem().wait(t.clone());
          Self::thread_yield()
        }
      }
    } else {
      Err(ERROR_INVARG)
    }
  }

  fn mem_alloc(asid: u16, va: usize, attr: usize) -> SyscallResult {
    let va = round_down(va, PAGE_SIZE);
    // if va >= CONFIG_USER_LIMIT {
    //   return Err(MemoryLimitError);
    // }
    let a = lookup_as(asid)?;
    let frame = crate::mm::page_pool::page_alloc().map_err(|_| ERROR_OOM)?;
    frame.zero();
    let attr = Entry::from(ArchPageTableEntry::from_pte(attr)).attribute().filter();
    let uf = crate::mm::Frame::from(frame);
    a.page_table().insert_page(va, uf, attr).map_err(|_| ERROR_INTERNAL)?;
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
    let attr = Entry::from(ArchPageTableEntry::from_pte(attr)).attribute().filter();
    if let Some(uf) = src_as.page_table().lookup_user_page(src_va) {
      dst_as.page_table().insert_page(dst_va, uf, attr).map_err(|_| ERROR_INTERNAL)?;
      Ok(Unit)
    } else {
      Err(ERROR_MEM_NOT_MAP)
    }
  }

  fn mem_unmap(asid: u16, va: usize) -> SyscallResult {
    let va = round_down(va, PAGE_SIZE);
    // if va >= CONFIG_USER_LIMIT {
    //   return Err(MemoryLimitError);
    // }
    let a = lookup_as(asid)?;
    a.page_table().remove_page(va).map_err(|_| ERROR_INTERNAL)?;
    Ok(Unit)
  }

  fn address_space_alloc() -> SyscallResult {
    let a = crate::lib::address_space::address_space_alloc()?;
    Ok(Single(a.asid() as usize))
  }

  fn thread_alloc(asid: u16, entry: usize, sp: usize, arg: usize) -> SyscallResult {
    let t = current_thread()?;
    let a = lookup_as(asid)?;
    let child_thread = crate::lib::thread::new_user(entry, sp, arg, a.clone(), Some(t.tid()));
    Ok(Single(child_thread.tid() as usize))
  }

  fn thread_set_status(tid: usize, status: usize) -> SyscallResult {
    use common::thread::*;
    let runnable = match status {
      THREAD_STATUS_NOT_RUNNABLE => false,
      THREAD_STATUS_RUNNABLE => true,
      _ => return Err(ERROR_INVARG)
    };
    match crate::lib::thread::thread_lookup(tid) {
      None => {}
      Some(t) => {
        if runnable {
          thread_wake(&t);
        } else {
          thread_sleep(&t);
        }
      }
    }
    Ok(Unit)
  }

  fn address_space_destroy(asid: u16) -> SyscallResult {
    let a = lookup_as(asid)?;
    address_space_destroy(a);
    Ok(Unit)
  }

  fn itc_receive() -> SyscallResult {
    let t = current_thread()?;
    t.ready_to_receive();
    if let Some(0) = t.is_serving() {
      t.ready_to_serve();
    }
    thread_sleep(&t);
    crate::lib::cpu::cpu().schedule();
    Ok(Unit)
  }

  fn itc_send(tid: Tid, a: usize, b: usize, c: usize, d: usize) -> SyscallResult {
    let t = current_thread()?;
    let target = crate::lib::thread::thread_lookup(tid).ok_or_else(|| ERROR_INVARG)?;
    if target.receive() {
      target.map_with_context(|ctx| {
        ctx.set_syscall_result(&SyscallResult::Ok(SyscallOutRegisters::Pentad(t.tid() as usize, a, b, c, d)));
      });
      thread_wake(&target);
      if let Some(caller) = t.is_serving() {
        if caller == target.tid() {
          t.ready_to_serve();
        }
      }
      Ok(Unit)
    } else {
      Err(ERROR_HOLD_ON)
    }
  }

  fn itc_call(tid: Tid, a: usize, b: usize, c: usize, d: usize) -> SyscallResult {
    let t = current_thread()?;
    let target = crate::lib::thread::thread_lookup(tid).ok_or_else(|| ERROR_INVARG)?;

    if target.serve(t.tid()) {
      target.map_with_context(|ctx| {
        ctx.set_syscall_result(&SyscallResult::Ok(SyscallOutRegisters::Pentad(t.tid() as usize, a, b, c, d)));
      });
      thread_wake(&target);
      t.ready_to_receive();
      thread_sleep(&t);
      crate::lib::cpu::cpu().schedule();
      Ok(Unit)
    } else {
      Err(ERROR_HOLD_ON)
    }
  }

  fn server_register(server_id: usize) -> SyscallResult {
    let t = current_thread()?;
    super::server::set(server_id, t.tid());
    Ok(Unit)
  }

  fn server_tid(server_id: usize) -> SyscallResult {
    match super::server::get(server_id) {
      None => {
        Err(ERROR_INVARG)
      },
      Some(tid) => {
        Ok(Single(tid as usize))
      }
    }
  }
}

pub fn syscall() {
  use common::syscall::*;
  let ctx = crate::lib::cpu::cpu().context_mut();
  let tid = cpu().running_thread().map(|x| {x.tid()}).unwrap_or_default();
  let arg = |i: usize| { ctx.syscall_argument(i) };
  let num = ctx.syscall_number();
  let result = match num {
    SYS_NULL => Syscall::null(),
    SYS_PUTC => Syscall::putc(arg(0) as u8 as char),
    SYS_GET_ASID => Syscall::get_asid(arg(0)),
    SYS_GET_TID => Syscall::get_tid(),
    SYS_THREAD_YIELD => Syscall::thread_yield(),
    SYS_THREAD_DESTROY => Syscall::thread_destroy(arg(0)),
    SYS_EVENT_WAIT => Syscall::event_wait(arg(0), arg(1)),
    SYS_MEM_ALLOC => Syscall::mem_alloc(arg(0) as u16, arg(1), arg(2)),
    SYS_MEM_MAP => Syscall::mem_map(arg(0) as u16, arg(1), arg(2) as u16, arg(3), arg(4)),
    SYS_MEM_UNMAP => Syscall::mem_unmap(arg(0) as u16, arg(1)),
    SYS_ADDRESS_SPACE_ALLOC => Syscall::address_space_alloc(),
    SYS_THREAD_ALLOC => Syscall::thread_alloc(arg(0) as u16, arg(1), arg(2), arg(3)),
    SYS_THREAD_SET_STATUS => Syscall::thread_set_status(arg(0), arg(1)),
    SYS_ADDRESS_SPACE_DESTROY => Syscall::address_space_destroy(arg(0) as u16),
    SYS_ITC_RECV => Syscall::itc_receive(),
    SYS_ITC_SEND => Syscall::itc_send(arg(0), arg(1), arg(2), arg(3), arg(4)),
    SYS_ITC_CALL => Syscall::itc_call(arg(0), arg(1), arg(2), arg(3), arg(4)),
    SYS_SERVER_REGISTER => Syscall::server_register(arg(0)),
    SYS_SERVER_TID => Syscall::server_tid(arg(0)),
    _ => {
      warn!("system call: unrecognized system call number");
      Err(ERROR_INVARG)
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
  // TODO: schedule happened overwrite ctx's value
  if num != SYS_ITC_RECV && num != SYS_THREAD_YIELD {
    ctx.set_syscall_result(&result);
  }
}