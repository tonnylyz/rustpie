use alloc::collections::BTreeMap;
use alloc::string::String;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::Relaxed;

use spin::Mutex;

use libtrusted::foreign_slice::ForeignSlice;
use libtrusted::wrapper::request_wrapper;
use microcall::{get_asid, get_tid};
use microcall::message::Message;

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
          microcall::address_space_destroy(p.asid).expect("process address space destroy failed");
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

#[inject::count_stmts]
#[inject::panic_inject]
#[inject::page_fault_inject]
fn pm(msg: Message, tid: usize) -> (usize, usize) {
  let asid = get_asid(tid).unwrap();
  match msg.a {
    cs::pm::action::SPAWN => {
      let length = msg.c;
      if length == 0 {
        return (cs::pm::result::INVARG, 0);
      }
      if length >= 128 {
        return (cs::pm::result::INVARG, 0);
      }
      let s = ForeignSlice::new(asid, msg.b, msg.c).unwrap();
      let cmd = s.local_slice();
      let cmd = core::str::from_utf8(cmd);
      if let Ok(cmd) = cmd {
        if let Ok((child_asid, tid)) = libtrusted::loader::spawn(cmd) {
          let pid = PROCESS_MANAGER.register(child_asid, tid, Some(asid as usize), String::from(cmd));
          microcall::thread_set_status(tid, common::thread::THREAD_STATUS_RUNNABLE).expect("pm start thread failed");
          (cs::pm::result::OK, pid)
        } else {
          (cs::pm::result::SPAWN_FAILED, 0)
        }
      } else {
        (cs::pm::result::INVARG, 0)
      }
    }
    cs::pm::action::WAIT => {
      let pid = msg.b;
      if PROCESS_MANAGER.poll_exit(pid) {
        (cs::pm::result::OK, 0)
      } else {
        (cs::pm::result::HOLD_ON, 0)
      }
    }
    cs::pm::action::PS => {
      PROCESS_MANAGER.ps();
      (cs::pm::result::OK, 0)
    }
    _ => {
      (cs::pm::result::INVARG, 0)
    }
  }
}

pub fn server() {
  info!("server started t{}", get_tid());
  microcall::server_register(common::server::SERVER_PM).unwrap();
  loop {
    let (client_tid, msg) = Message::receive().unwrap();
    let (a, b) = request_wrapper(pm, msg, client_tid).unwrap();
    let result = Message::new(a, b, 0, 0);
    let _ = result.send_to(client_tid);
  }
}
