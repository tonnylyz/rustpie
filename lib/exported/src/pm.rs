use alloc::boxed::Box;
use common::PAGE_SIZE;
use crate::server::*;

#[repr(align(4096))]
struct Page {
  bytes: [u8; PAGE_SIZE],
}

pub fn exec(path: &str, arg: usize) -> Result<(), &'static str> {
  let mut page = Box::new(Page { bytes: [0; PAGE_SIZE] });
  let path = path.as_bytes();
  for i in 0..path.len() {
    page.bytes[i] = path[i];
  }

  let result = call(common::server::SERVER_PM, Message(
    1, page.as_ref() as *const _ as usize, arg, 0
  ))?;
  match result.0 {
    0 => Ok(()),
    _ => Err("exec failed"),
  }
}
