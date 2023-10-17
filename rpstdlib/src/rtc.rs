pub use rpabi::time::RtcTime;
use rpsyscall::message::Message;

pub fn timestamp() -> Result<u64, &'static str> {
  let result = Message::new(0, 0, 0, 0)
    .call(rpabi::server::SERVER_RTC)
    .map_err(|_| "server call failed")?;
  Ok(result.a as u64)
}
