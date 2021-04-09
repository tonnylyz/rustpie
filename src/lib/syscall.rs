use SyscallValue::*;

use crate::arch::{ArchPageTableEntry, ArchPageTableEntryTrait, ContextFrameTrait, PAGE_SIZE, Address};
use crate::config::CONFIG_USER_LIMIT;
use crate::lib::round_down;
use crate::lib::address_space::AddressSpace;
use crate::lib::core::{CoreTrait, current};
use crate::lib::event::Event;
use crate::lib::page_table::{Entry, PageTableEntryAttrTrait, PageTableTrait};

use self::Error::*;
use crate::lib::thread::Status::{TsNotRunnable, TsRunnable};
use core::mem::size_of;

#[derive(Debug)]
pub enum Error {
  InvalidArgumentError = 1,
  _OutOfProcessError,
  OutOfMemoryError,
  MemoryLimitError,
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

impl Into<SyscallResult> for SyscallValue {
  fn into(self) -> SyscallResult {
    Ok(self)
  }
}

#[derive(Debug)]
pub enum SyscallValue {
  Unit,
  U32(u32),
  U16(u16),
  ISize(isize),
  USize(usize),
}

pub type SyscallResult = Result<SyscallValue, Error>;

static SYSCALL_NAMES: [&str; 17] = [
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
];

pub trait SyscallTrait {
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
  fn itc_send(tid: u16, a: usize, b: usize, c: usize, d: usize) -> SyscallResult;
}

pub struct Syscall;

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

const OK: SyscallResult = Ok(SyscallValue::Unit);

impl SyscallTrait for Syscall {
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
      current_thread.destroy();
      Syscall::thread_yield()
    } else {
      match crate::lib::thread::lookup(tid) {
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
      match crate::lib::interrupt::INTERRUPT_WAIT.add(current().running_thread().unwrap().clone(), event) {
        Ok(_) => { return OK; }
        Err(_) => { return Err(InternalError); }
      }
    }
    OK
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
    OK
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
      OK
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
    let a = t.address_space().unwrap();
    let child_thread = crate::lib::thread::new_user(entry, sp, arg, a.clone(), current().running_thread());
    child_thread.set_status(crate::lib::thread::Status::TsRunnable);
    U16(child_thread.tid()).into()
  }

  fn thread_set_status(tid: u16, status: crate::lib::thread::Status) -> SyscallResult {
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

  fn itc_receive(msg_ptr: usize)-> SyscallResult {
    let t = current().running_thread().ok_or_else(|| InternalError)?;
    t.set_msg_ptr(msg_ptr);
    t.set_status(TsNotRunnable);
    crate::lib::core::current().schedule();

    OK
  }

  fn itc_send(tid: u16, a: usize, b: usize, c: usize, d: usize) -> SyscallResult {
    let t = current().running_thread().ok_or_else(|| InternalError)?;
    let target = crate::lib::thread::lookup(tid).ok_or_else(|| InvalidArgumentError)?;
    if t.address_space() != target.address_space() {
      return Err(PermissionDenied);
    }
    let msg_ptr = target.msg_ptr().ok_or_else(|| NotReceivingError)?;
    let addr_space =  t.address_space().ok_or_else(|| InternalError)?;
    let pt = addr_space.page_table();
    let uf = pt.lookup_user_page(msg_ptr).ok_or_else(|| PermissionDenied)?;

    let va = uf.pa().pa2kva() + (msg_ptr % PAGE_SIZE);
    if va % size_of::<usize>() != 0 {
      // check alignment
      println!("msg_ptr does not align");
      return Err(InternalError);
    }
    if va + 4 * size_of::<usize>() > uf.pa().pa2kva() + PAGE_SIZE {
      // must inside one page
      println!("msg_ptr cross page border");
      return Err(InternalError);
    }
    unsafe {
      ((va + 0 * size_of::<usize>()) as *mut usize).write_volatile(a);
      ((va + 1 * size_of::<usize>()) as *mut usize).write_volatile(b);
      ((va + 2 * size_of::<usize>()) as *mut usize).write_volatile(c);
      ((va + 3 * size_of::<usize>()) as *mut usize).write_volatile(d);
    }
    if target.status() != TsNotRunnable {
      println!("recv thread runnable?");
      return Err(InternalError);
    }
    let mut ctx = target.context();
    ctx.set_syscall_return_value(t.tid() as usize);
    target.set_context(ctx);
    target.set_status(TsRunnable);
    Ok(ISize(0))
  }
}

pub fn syscall() {
  let ctx = crate::lib::core::current().context_mut();
  let tid = current().running_thread().unwrap().tid();
  let arg = |i: usize| { ctx.syscall_argument(i) };
  let num = ctx.syscall_number();
  let scr = match num {
    0 => Syscall::null(),
    1 => Syscall::putc(arg(0) as u8 as char),
    2 => Syscall::get_asid(arg(0) as u16),
    3 => Syscall::get_tid(),
    4 => Syscall::thread_yield(),
    5 => Syscall::thread_destroy(arg(0) as u16),
    6 => Syscall::event_handler(arg(0) as u16, arg(1), arg(2), arg(3)),
    7 => Syscall::mem_alloc(arg(0) as u16, arg(1), arg(2)),
    8 => Syscall::mem_map(arg(0) as u16, arg(1), arg(2) as u16, arg(3), arg(4)),
    9 => Syscall::mem_unmap(arg(0) as u16, arg(1)),
    10 => Syscall::address_space_alloc(),
    11 => Syscall::thread_alloc(arg(0), arg(1), arg(2)),
    12 => {
      match arg(1) {
        1 => { Syscall::thread_set_status(arg(0) as u16, TsRunnable) }
        2 => { Syscall::thread_set_status(arg(0) as u16, TsNotRunnable) }
        _ => { Err(super::syscall::Error::InvalidArgumentError) }
      }
    }
    13 => Syscall::ipc_receive(arg(0)),
    14 => Syscall::ipc_can_send(arg(0) as u16, arg(1), arg(2), arg(3)),
    15 => Syscall::itc_receive(arg(0)),
    16 => Syscall::itc_send(arg(0) as u16, arg(1), arg(2), arg(3), arg(4)),
    _ => {
      println!("system call: unrecognized system call number");
      Err(super::syscall::Error::InvalidArgumentError)
    }
  };
  match scr {
    Ok(val) => {

      match val {
        SyscallValue::Unit => {}
        SyscallValue::U32(u) => {
          ctx.set_syscall_return_value(u as usize);
        }
        SyscallValue::U16(u) => {
          ctx.set_syscall_return_value(u as usize);
        }
        SyscallValue::ISize(i) => {
          ctx.set_syscall_return_value(i as usize);
        }
        SyscallValue::USize(u) => {
          ctx.set_syscall_return_value(u as usize);
        }
      }
      // if num != 1 {
      //   println!("#{}\t{} t{} Ok {:x}", num, SYSCALL_NAMES[num], tid, ctx.syscall_argument(0));
      // }
    }
    Err(err) => {
      // println!("#{}\t{} t{} Err {:x?}", num, SYSCALL_NAMES[num], tid, err);
      ctx.set_syscall_return_value(usize::MAX);
    }
  }
}