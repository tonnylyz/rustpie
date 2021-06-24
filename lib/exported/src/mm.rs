
pub fn mem_alloc(va: usize) -> Result<(), &'static str> {
  let server_tid = microcall::server_tid_wait(common::server::SERVER_MM);
  loop {
    match microcall::itc_call(server_tid, 1, va, 0, 0) {
      Ok((_, r, _, _, _)) => {
        if r == 0 {
          break Ok(())
        } else {
          break Err("mem_alloc failed")
        }
      }
      Err(_) => {
        microcall::thread_yield()
      }
    }
  }
}
