use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::Relaxed;

use common::PAGE_SIZE;
use spin::Mutex;

use libtrusted::foreign_slice::ForeignSlice;
use libtrusted::mm::{Entry, EntryLike};
use microcall::{get_asid, get_tid, mem_map, mem_unmap};
use microcall::message::Message;

pub const PM_ACTION_SPAWN: usize = 1;
pub const PM_ACTION_WAIT: usize = 2;

pub const PM_RESULT_OK: usize = 0;
pub const PM_RESULT_INVARG: usize = 1;
pub const PM_RESULT_NOMEM: usize = 2;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum ProcessStatus {
  Running,
  Exited,
}

static PID_ALLOCATOR: AtomicUsize = AtomicUsize::new(0x8000_0000);

struct Process {
  pid: usize,
  parent: Option<usize>,
  asid: u16,
  main_tid: usize,
  status: ProcessStatus,
}

struct ProcessManager {
  list: Mutex<Vec<Process>>,
}

impl ProcessManager {
  const fn new() -> Self {
    ProcessManager {
      list: Mutex::new(Vec::new())
    }
  }

  fn register(&self, asid: u16, tid: usize, parent: Option<usize>) -> usize {
    let pid = PID_ALLOCATOR.fetch_add(1, Relaxed);
    let p = Process {
      pid,
      parent,
      asid,
      main_tid: tid,
      status: ProcessStatus::Running
    };
    let mut vec = self.list.lock();
    vec.push(p);
    pid as usize
  }

  fn status(&self, pid: usize) -> Option<ProcessStatus> {
    let vec = self.list.lock();
    for p in vec.iter() {
      if p.pid == pid {
        return Some(p.status);
      }
    }
    None
  }

  fn set_status_by_tid(&self, tid: usize, status: ProcessStatus) -> Result<(), ()> {
    let mut vec = self.list.lock();
    for p in vec.iter_mut() {
      if p.status == ProcessStatus::Running && p.main_tid == tid {
        p.status = status;
        return Ok(());
      }
    }
    Err(())
  }

  fn main_tid_to_asid(&self, tid: usize) -> Option<u16> {
    let vec = self.list.lock();
    for p in vec.iter() {
      if p.main_tid == tid {
        return Some(p.asid);
      }
    }
    None
  }
}

static PROCESS_MANAGER: ProcessManager = ProcessManager::new();

fn process_request(asid: u16, msg: &Message) -> Result<(usize, usize, usize), usize> {
  match msg.a {
    PM_ACTION_SPAWN => {
      let length = msg.c;
      if length == 0 {
        return Err(PM_RESULT_INVARG);
      }
      if length >= 128 {
        return Err(PM_RESULT_INVARG);
      }
      let s = ForeignSlice::new(asid, msg.b, msg.c).unwrap();
      let cmd = s.local_slice();
      let cmd = core::str::from_utf8(cmd);
      if let Ok(cmd) = cmd {
        if let Ok((child_asid, tid)) = libtrusted::loader::spawn(cmd) {
          let pid = PROCESS_MANAGER.register(child_asid, tid, Some(asid as usize));
          Ok((pid, 0, 0))
        } else {
          Err(PM_RESULT_NOMEM)
        }
      } else {
        Err(PM_RESULT_INVARG)
      }
    }
    PM_ACTION_WAIT => {
      let pid = msg.b;
      if let Some(status) = PROCESS_MANAGER.status(pid) {
        match status {
          ProcessStatus::Running => Ok((0, 0, 0)),
          ProcessStatus::Exited => Ok((1, 0, 0)),
        }
      } else {
        Err(PM_RESULT_INVARG)
      }
    }
    _ => {
      Err(PM_RESULT_INVARG)
    }
  }
}

pub fn server() {
  info!("server started t{}", get_tid());
  microcall::server_register(common::server::SERVER_PM).unwrap();
  loop {
    let (tid, msg) = Message::receive().unwrap();
    trace!("t{}: {:x?}", tid, msg);
    let asid = get_asid(tid).unwrap();
    let mut result = Message::default();
    match process_request(asid, &msg) {
      Ok((b, c, d)) => {
        result.a = PM_RESULT_OK;
        result.b = b;
        result.c = c;
        result.d = d;
      }
      Err(e) => {
        result.a = e;
      }
    }
    let _ = result.send_to(tid);
  }
}

pub fn event_server() {
  loop {
    if let Ok(tid) = microcall::event_wait(common::event::EVENT_THREAD_EXIT, 0) {
      PROCESS_MANAGER.set_status_by_tid(tid, ProcessStatus::Exited);
      if let Some(asid) = PROCESS_MANAGER.main_tid_to_asid(tid) {
        microcall::address_space_destroy(asid);
      }
    } else {
      microcall::thread_yield();
    }
  }
}
