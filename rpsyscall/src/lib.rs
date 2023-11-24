#![no_std]

use rpabi::syscall::*;

use arch::*;

#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64.rs"]
mod arch;

#[cfg(target_arch = "riscv64")]
#[path = "arch/riscv64.rs"]
mod arch;

#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64.rs"]
mod arch;

/// Null system call passing one dummy argument
pub fn null(dummy: usize) {
  let _ = syscall_1_0(SYS_NULL, dummy);
}

/// Put raw character to system console
///
/// # Arguments
///
/// * `c` - ASCII u8 of a raw character
pub fn putraw(c: u8) {
  let _ = syscall_1_0(SYS_PUTC, c as usize);
}

/// Put character to system console
///
/// # Arguments
///
/// * `c` - Rust char of a character
pub fn putc(c: char) {
  putraw(c as u8);
}

/// Get AddressSpace identifier by identifier of a thread
///
/// # Arguments
///
/// * `tid` - Thread Identifier of the thread
pub fn get_asid(tid: usize) -> Result<u16, Error> {
  syscall_1_1(SYS_GET_ASID, tid).map(|asid| asid as u16)
}

/// Get identifier of current thread
pub fn get_tid() -> usize {
  syscall_0_1(SYS_GET_TID).unwrap()
}

/// Voluntarily yield CPU time to other threads
///
/// This syscall will result in a thread switch
pub fn thread_yield() {
  let _ = syscall_0_0(SYS_THREAD_YIELD);
}

/// Destroy or terminate a thread
///
/// This syscall should not return
///
/// # Arguments
///
/// * `tid` - identifier of the thread to destroy, set it `0` to destroy current thread
pub fn thread_destroy(tid: usize) {
  syscall_1_0(SYS_THREAD_DESTROY, tid).unwrap()
}

/// Wait for kernel event
///
/// There are two types of event in rustpi:
/// * Interrupt
/// * Thread exit event
///
/// User-space use this syscall to sleep until the wanted event happens
///
/// # Arguments
///
/// * `event_type` - event types defined in `rpabi::event::`
/// * `event_num` - for interrupt type: event_num is the interrupt number of system main interrupt controller (e.g., GIC)
/// ; for thread exit event: event_num is the identifier of the thread being waited to exit
pub fn event_wait(event_type: usize, event_num: usize) -> Result<usize, Error> {
  syscall_2_1(SYS_EVENT_WAIT, event_type, event_num)
}

/// Allocate physical memory page
///
/// # Arguments
///
/// * `asid` - identifier of AddressSpace where the allocation happens. Set it `0` to allocate on current AddressSpace
/// * `va` - virtual address to access the physical memory page
/// * `attr` - virtual page attributes defined in `rpabi::syscall::mm::EntryAttribute`. This value is arch-independent.
pub fn mem_alloc(asid: u16, va: usize, attr: usize) -> Result<(), Error> {
  syscall_3_0(SYS_MEM_ALLOC, asid as usize, va, attr)
}

/// Share a physical memory page across two AddressSpace
///
/// # Arguments
///
/// * `src_asid` - identifier of the source AddressSpace the physical page currently allocated in
/// * `src_va` - virtual address of the page in source AddressSpace
/// * `dst_asid` - identifier of the destination AddressSpace the new mapping happens in
/// * `dst_va` - virtual address of the page in destination AddressSpace
/// * `attr` - virtual page attributes defined in `rpabi::syscall::mm::EntryAttribute`. This value is arch-independent.
pub fn mem_map(
  src_asid: u16,
  src_va: usize,
  dst_asid: u16,
  dst_va: usize,
  attr: usize,
) -> Result<(), Error> {
  syscall_5_0(
    SYS_MEM_MAP,
    src_asid as usize,
    src_va,
    dst_asid as usize,
    dst_va,
    attr,
  )
}

/// Unmap virtual page at `va` in an AddressSpace
///
/// # Arguments
///
/// * `asid` - identifier of AddressSpace where the un-mapping happens. Set it `0` for current AddressSpace
/// * `va` - virtual address of the memory page to be un-mapped
pub fn mem_unmap(asid: u16, va: usize) -> Result<(), Error> {
  syscall_2_0(SYS_MEM_UNMAP, asid as usize, va)
}

/// Create a new AddressSpace
///
/// This syscall requests kernel to allocate new AddressSpace. It returns identifier of the newly created AddressSpace
pub fn address_space_alloc() -> Result<u16, Error> {
  syscall_0_1(SYS_ADDRESS_SPACE_ALLOC).map(|asid| asid as u16)
}

/// Create a new thread in specific AddressSpace
///
/// The new thread is not runnable after this call. Use `thread_set_status` to wake it up.
///
/// # Arguments
///
/// * `asid` - identifier of the AddressSpace the new thread uses
/// * `entry` - entry point or initial program counter(PC)/instruction pointer(IP) the thread
/// * `sp` - initial stack pointer value. It typically points to stack top
/// * `arg` - argument passed to the thread
pub fn thread_alloc(asid: u16, entry: usize, sp: usize, arg: usize) -> Result<usize, Error> {
  syscall_4_1(SYS_THREAD_ALLOC, asid as usize, entry, sp, arg)
}

/// Explicitly set thread status
///
/// Current thread need be parent of the target thread.
///
/// # Arguments
///
/// * `tid` - identifier of the target thread.
/// * `status` - status defined in `rpabi::thread::`. Valid values are `THREAD_STATUS_RUNNABLE` or `THREAD_STATUS_NOT_RUNNABLE`
pub fn thread_set_status(tid: usize, status: usize) -> Result<(), Error> {
  syscall_2_0(SYS_THREAD_SET_STATUS, tid, status)
}

/// Destroy an AddressSpace
///
/// # Arguments
///
/// * `asid` - identifier of the AddressSpace to be destroyed
pub fn address_space_destroy(asid: u16) -> Result<(), Error> {
  syscall_1_0(SYS_ADDRESS_SPACE_DESTROY, asid as usize)
}

/// Actively receive message from other threads
///
/// This syscall returns when message is received and represented in return values
pub fn itc_receive() -> Result<(usize, usize, usize, usize, usize), Error> {
  syscall_0_5(SYS_ITC_RECV)
}

/// Send a message to a thread
///
/// This syscall doesn't block and it returns error when the target thread isn't in `receive` status.
///
/// # Arguments
///
/// * `tid` - identifier of the target thread
/// * `a` - first component of the message
/// * `b` - second component of the message
/// * `c` - third component of the message
/// * `d` - fourth component of the message
pub fn itc_send(tid: usize, a: usize, b: usize, c: usize, d: usize) -> Result<(), Error> {
  syscall_5_0(SYS_ITC_SEND, tid as usize, a, b, c, d)
}

/// Send a message to a thread and wait for a reply message
///
/// This syscall make the current thread blocked until it gets reply. It's typically used in client-server invocation.
///
/// # Arguments
///
/// * `tid` - identifier of the target thread
/// * `a` - first component of the message
/// * `b` - second component of the message
/// * `c` - third component of the message
/// * `d` - fourth component of the message
pub fn itc_call(
  tid: usize,
  a: usize,
  b: usize,
  c: usize,
  d: usize,
) -> Result<(usize, usize, usize, usize, usize), Error> {
  syscall_5_5(SYS_ITC_CALL, tid as usize, a, b, c, d)
}

/// Register current thread as a server specified by server_id
///
/// # Arguments
///
/// * `server_id` - identifiers defined in `rpabi::server`
pub fn server_register(server_id: usize) -> Result<(), Error> {
  syscall_1_0(SYS_SERVER_REGISTER, server_id)
}

/// Get thread identifier of a server
///
/// This syscall return an error when target server isn't running
///
/// # Arguments
///
/// * `server_id` - identifier of the target server
fn server_tid(server_id: usize) -> Result<usize, Error> {
  syscall_1_1(SYS_SERVER_TID, server_id)
}

/// Wait for server running and return its thread identifier
///
/// # Arguments
///
/// * `server_id` - identifier of the target server
pub fn server_tid_wait(server_id: usize) -> usize {
  loop {
    if let Ok(tid) = server_tid(server_id) {
      break tid;
    }
    thread_yield();
  }
}

/// Set exception handler for current AddressSpace
///
/// Upon a user-space exception happens, the thread has exception will jump the specific handler with stack setup by kernel
///
/// # Arguments
///
/// * `handler` - address of the handler function
pub fn set_exception_handler(handler: usize) -> Result<(), Error> {
  syscall_1_0(SYS_SET_EXCEPTION_HANDLER, handler)
}

/// Get an input character from system console
///
/// This syscall isn't implemented for all platforms
pub fn getc() -> Result<u8, Error> {
  syscall_0_1(SYS_GETC).map(|c| c as u8)
}

/// Reply a message to the target thread and actively receive message from other threads immediately
///
/// This syscall is a combo of `itc_send` and `itc_receive`
///
/// # Arguments
///
/// * `tid` - identifier of the target thread
/// * `a` - first component of the message
/// * `b` - second component of the message
/// * `c` - third component of the message
/// * `d` - fourth component of the message
pub fn itc_reply_recv(
  tid: usize,
  a: usize,
  b: usize,
  c: usize,
  d: usize,
) -> Result<(usize, usize, usize, usize, usize), Error> {
  syscall_5_5(SYS_REPLY_RECV, tid as usize, a, b, c, d)
}

/// Helper struct for inter-thread-communication
pub mod message {

  #[repr(C)]
  #[derive(Copy, Clone, Debug, Default)]
  /// Inter-thread-communication Message
  ///
  /// In rustpi, ITC message contains four components and they are all passed by registers.
  ///
  /// Usage of each components can be defined as per each server/client. It can be found in `rpservapi` crate.
  pub struct Message {
    /// first component of the message
    pub a: usize,
    /// second component of the message
    pub b: usize,
    /// third component of the message
    pub c: usize,
    /// fourth component of the message
    pub d: usize,
  }

  impl Message {
    /// Construct new message
    pub fn new(a: usize, b: usize, c: usize, d: usize) -> Self {
      Message { a, b, c, d }
    }

    /// Current thread sleeps until received a message
    pub fn receive() -> Result<(usize, Self), super::Error> {
      super::itc_receive().map(|(tid, a, b, c, d)| (tid, Message { a, b, c, d }))
    }

    /// Send message to a thread (non-blocked)
    pub fn send_to(&self, tid: usize) -> Result<(), super::Error> {
      super::itc_send(tid, self.a, self.b, self.c, self.d)
    }

    /// Reply message to a thread and start to receive message at once
    pub fn reply_recv(&self, tid: usize) -> Result<(usize, Self), super::Error> {
      super::itc_reply_recv(tid, self.a, self.b, self.c, self.d)
        .map(|(tid, a, b, c, d)| (tid, Message { a, b, c, d }))
    }

    /// Send message to a server and wait for its reply
    pub fn call(&self, server_id: usize) -> Result<Self, super::Error> {
      use rpabi::syscall::error::ERROR_HOLD_ON;
      let server_tid = super::server_tid_wait(server_id);
      loop {
        match super::itc_call(server_tid, self.a, self.b, self.c, self.d) {
          Ok((_, a, b, c, d)) => {
            break Ok(Message { a, b, c, d });
          }
          Err(ERROR_HOLD_ON) => {
            super::thread_yield();
          }
          Err(e) => {
            break Err(e);
          }
        }
      }
    }
  }
}
