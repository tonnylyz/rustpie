use microcall::message::Message;

pub fn exec(cmd: &str) -> Result<usize, &'static str> {
  let result = Message::new(
    1, cmd.as_ptr() as usize, cmd.len(), 0
  ).call(common::server::SERVER_PM).map_err(|_| "server call failed")?;
  match result.a {
    0 => Ok(result.b),
    _ => Err("exec failed"),
  }
}

pub fn wait(pid: usize) {
  loop {
    let result = Message::new(
      2, pid, 0, 0
    ).call(common::server::SERVER_PM).expect("server call failed");
    match result.a {
      0 => {
        match result.b {
          0 => microcall::thread_yield(),
          1 => break,
          _ => panic!("wait result invalid"),
        }
      },
      _ => panic!("wait failed"),
    }
  }
}

pub fn ps() {
  Message::new(
    3, 0, 0, 0
  ).call(common::server::SERVER_PM);
}
