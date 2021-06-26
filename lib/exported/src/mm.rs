use microcall::message::Message;

pub fn page_alloc(va: usize) -> Result<(), &'static str> {
  let result = Message::new(
    1, va, 0, 0
  ).call(common::server::SERVER_MM).map_err(|_| "server call failed")?;
  match result.a {
    0 => Ok(()),
    _ => Err("page_alloc failed"),
  }
}
