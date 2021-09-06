use alloc::collections::BTreeMap;
use common::syscall::error::ERROR_INVARG;
use spin::Mutex;
use crate::lib::thread::Tid;
use super::{Result, SyscallOutRegisters::*};

#[inline(never)]
pub fn server_register(server_id: usize) -> Result {
  let t = super::current_thread()?;
  set(server_id, t.tid());
  Ok(Unit)
}

#[inline(never)]
pub fn server_tid(server_id: usize) -> Result {
  match get(server_id) {
    None => {
      Err(ERROR_INVARG)
    }
    Some(tid) => {
      Ok(Single(tid as usize))
    }
  }
}

static SERVER_MAP: Mutex<BTreeMap<usize, Tid>> = Mutex::new(BTreeMap::new());

fn get(server_id: usize) -> Option<Tid> {
  let map = SERVER_MAP.lock();
  map.get(&server_id).map(|x| *x)
}

fn set(server_id: usize, tid: Tid) {
  let mut map = SERVER_MAP.lock();
  if let Some(t) = map.get_mut(&server_id) {
    *t = tid;
  } else {
    map.insert(server_id, tid);
  }
}
