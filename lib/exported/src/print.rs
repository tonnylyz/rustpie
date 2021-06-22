use core::fmt;
use spin::Mutex;

use microcall::{putc, get_tid};

struct Writer;

static WRITER: Mutex<Writer> = Mutex::new(Writer);

impl fmt::Write for Writer {
  fn write_str(&mut self, s: &str) -> fmt::Result {
    for c in s.chars() {
      putc(c);
    }
    Ok(())
  }
}

pub fn print_arg(args: fmt::Arguments) {
  use core::fmt::Write;
  let mut writer = WRITER.lock();
  writer.write_fmt(args).unwrap();
}

#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
  loop {}
}
