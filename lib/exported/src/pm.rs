use microcall::message::Message;

pub fn exec(path: &str, arg: usize) -> Result<(), &'static str> {
  let result = Message::new(
    1, path.as_ptr() as usize, path.len(), arg
  ).call(common::server::SERVER_PM).map_err(|_| "server call failed")?;
  match result.a {
    0 => Ok(()),
    _ => Err("exec failed"),
  }
}
