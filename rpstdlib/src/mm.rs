use rpsyscall::message::Message;

pub fn page_alloc(va: usize) -> Result<(), &'static str> {
  let result = Message::new(
    rpservapi::mm::action::ALLOC, va, 0, 0,
  ).call(rpabi::server::SERVER_MM).map_err(|_| "server call failed")?;
  match result.a {
    rpservapi::mm::result::OK => Ok(()),
    _ => Err("page_alloc failed"),
  }
}
