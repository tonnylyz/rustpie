use crate::server::*;

pub fn page_alloc(va: usize) -> Result<(), &'static str> {
  let result = call(common::server::SERVER_MM, Message(
    1, va, 0, 0
  ))?;
  match result.0 {
    0 => Ok(()),
    _ => Err("page_alloc failed"),
  }
}
