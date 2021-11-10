use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::Relaxed;


use spin::Mutex;

use libtrusted::foreign_slice::ForeignSlice;

use microcall::{get_asid, get_tid};
use microcall::message::Message;

pub const PM_ACTION_SPAWN: usize = 1;
pub const PM_ACTION_WAIT: usize = 2;
pub const PM_ACTION_PS: usize = 3;

pub const PM_RESULT_OK: usize = 0;
pub const PM_RESULT_INVARG: usize = 1;
pub const PM_RESULT_NOMEM: usize = 2;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum ProcessStatus {
  Running,
  Exited,
}

static PID_ALLOCATOR: AtomicUsize = AtomicUsize::new(200);

struct Process {
  pid: usize,
  parent: Option<usize>,
  asid: u16,
  main_tid: usize,
  status: ProcessStatus,
  command: String,
}

struct ProcessManager {
  list: Mutex<BTreeMap<usize, Process>>,
}

impl ProcessManager {
  const fn new() -> Self {
    ProcessManager {
      list: Mutex::new(BTreeMap::new())
    }
  }

  fn register(&self, asid: u16, tid: usize, parent: Option<usize>, command: String) -> usize {
    let pid = PID_ALLOCATOR.fetch_add(1, Relaxed);
    let p = Process {
      pid,
      parent,
      asid,
      main_tid: tid,
      status: ProcessStatus::Running,
      command,
    };
    let mut map = self.list.lock();
    map.insert(pid, p);
    pid as usize
  }

  fn poll_exit(&self, pid: usize) -> bool {
    let mut map = self.list.lock();
    if let Some(p) = map.get_mut(&pid) {
      if p.status == ProcessStatus::Exited {
        return true;
      } else {
        if let Ok(_) = microcall::event_wait(common::event::EVENT_THREAD_EXIT, p.main_tid) {
          p.status = ProcessStatus::Exited;
          microcall::address_space_destroy(p.asid);
          return true;
        } else {
          return false;
        }
      }
    }
    false
  }

  fn ps(&self) {
    let map = self.list.lock();
    println!("PID\t\tSTATUS\t\tTID\t\tPASID\t\tASID\t\tCOMMAND");
    for pid in map.keys() {
      if let Some(p) = map.get(pid) {
        println!("{}\t\t{:?}\t\t{}\t\t{:?}\t\t{}\t\t{}",
                 p.pid,
                 p.status,
                 p.main_tid,
                 p.parent,
                 p.asid,
                 p.command,
        );
      }
    }
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
          let pid = PROCESS_MANAGER.register(child_asid, tid, Some(asid as usize), String::from(cmd));
          microcall::thread_set_status(tid, common::thread::THREAD_STATUS_RUNNABLE);
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
      if PROCESS_MANAGER.poll_exit(pid) {
        Ok((1, 0, 0))
      } else {
        Ok((0, 0, 0))
      }
    }
    PM_ACTION_PS => {
      PROCESS_MANAGER.ps();
      Ok((0, 0, 0))
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
