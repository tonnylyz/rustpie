use microcall::message::Message;

pub fn exec(cmd: &str) -> Result<(), &'static str> {
  let result = Message::new(
    1, cmd.as_ptr() as usize, cmd.len(), 0
  ).call(common::server::SERVER_PM).map_err(|_| "server call failed")?;
  match result.a {
    0 => Ok(()),
    _ => Err("exec failed"),
  }
}
