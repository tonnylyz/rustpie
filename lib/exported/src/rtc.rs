use microcall::message::Message;

pub fn timestamp() -> Result<u64, &'static str> {
  let result = Message::new(
    0, 0, 0, 0
  ).call(common::server::SERVER_RTC).map_err(|_| "server call failed")?;
  Ok(result.a as u64)
}
