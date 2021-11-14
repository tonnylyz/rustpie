use microcall::message::Message;

pub fn page_alloc(va: usize) -> Result<(), &'static str> {
  let result = Message::new(
    cs::mm::action::ALLOC, va, 0, 0,
  ).call(common::server::SERVER_MM).map_err(|_| "server call failed")?;
  match result.a {
    cs::mm::result::OK => Ok(()),
    _ => Err("page_alloc failed"),
  }
}
