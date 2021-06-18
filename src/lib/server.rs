use alloc::collections::BTreeMap;
use crate::lib::thread::Tid;
use spin::Mutex;

static SERVER_MAP: Mutex<BTreeMap<usize, Tid>> = Mutex::new(BTreeMap::new());

pub fn get(server_id: usize) -> Option<Tid> {
  let map = SERVER_MAP.lock();
  map.get(&server_id).map(|x| *x)
}

pub fn set(server_id: usize, tid: Tid) {
  let mut map = SERVER_MAP.lock();
  if let Some(t) = map.get_mut(&server_id) {
    *t = tid;
  } else {
    map.insert(server_id, tid);
  }
}
