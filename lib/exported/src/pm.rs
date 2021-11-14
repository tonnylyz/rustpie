use microcall::message::Message;

pub fn exec(cmd: &str) -> Result<usize, &'static str> {
  let result = Message::new(
    cs::pm::action::SPAWN, cmd.as_ptr() as usize, cmd.len(), 0,
  ).call(common::server::SERVER_PM).map_err(|_| "server call failed")?;
  match result.a {
    cs::pm::result::OK => Ok(result.b),
    _ => Err("exec failed"),
  }
}

pub fn wait(pid: usize) {
  loop {
    let result = Message::new(
      cs::pm::action::WAIT, pid, 0, 0,
    ).call(common::server::SERVER_PM).expect("server call failed");
    match result.a {
      cs::pm::result::OK => {
        break;
      }
      cs::pm::result::HOLD_ON => {
        microcall::thread_yield();
      }
      _ => panic!("wait failed"),
    }
  }
}

pub fn ps() {
  let _ = Message::new(
    cs::pm::action::PS, 0, 0, 0,
  ).call(common::server::SERVER_PM);
}
