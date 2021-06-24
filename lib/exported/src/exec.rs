use alloc::boxed::Box;
use common::PAGE_SIZE;

#[repr(align(4096))]
struct Page {
  bytes: [u8; PAGE_SIZE],
}

pub fn exec(path: &str, arg: usize) -> Result<(), &'static str> {
  let server_tid = microcall::server_tid_wait(common::server::SERVER_PM);
  let mut page = Box::new(Page { bytes: [0; PAGE_SIZE] });
  let path = path.as_bytes();
  for i in 0..path.len() {
    page.bytes[i] = path[i];
  }
  loop {
    match microcall::itc_call(server_tid, 1, page.as_ref() as *const _ as usize, arg, 0) {
      Ok((_, r, _, _, _)) => {
        if r == 0 {
          break Ok(())
        } else {
          break Err("exec failed")
        }
      }
      Err(_) => {
        microcall::thread_yield()
      }
    }
  }
}
