use alloc::string::String;
use alloc::vec::Vec;

use spin::Mutex;

use rpsyscall::message::Message;

pub fn getchar() -> u8 {
  loop {
    let result = Message::default().call(rpabi::server::SERVER_TERMINAL).unwrap();
    match result.a as u8 {
      0 => rpsyscall::thread_yield(),
      8 | 127 => break 127, // backspace
      b'\r' | 32..=126 => { // carriage return or visible
        let c = result.a as u8;
        print!("{}", c as char);
        break c;
      }
      _ => continue,
    }
  }
}

pub fn getline() -> String {
  let mut v = Vec::new();
  loop {
    let c = getchar();
    if c == b'\r' {
      break;
    }
    if c == 127 {
      if !v.is_empty() {
        rpsyscall::putraw(c);
      }
      v.pop();
      continue;
    }
    v.push(c);
  }
  String::from_utf8(v).expect("getline failed!")
}


struct Writer;

static WRITER: Mutex<Writer> = Mutex::new(Writer);

impl core::fmt::Write for Writer {
  fn write_str(&mut self, s: &str) -> core::fmt::Result {
    for c in s.chars() {
      rpsyscall::putc(c);
    }
    Ok(())
  }
}

pub fn print_arg(args: core::fmt::Arguments) {
  use core::fmt::Write;
  let mut writer = WRITER.lock();
  writer.write_fmt(args).unwrap();
}
