use core::fmt;
use spin::Mutex;

use crate::microcall::{putc, thread_destroy};

macro_rules! print {
    ($($arg:tt)*) => ($crate::print::print_arg(format_args!($($arg)*)));
}

macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => ({
        $crate::print::print_arg(format_args_nl!($($arg)*));
    })
}

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
  if let Some(m) = info.message() {
    if let Some(l) = info.location() {
      println!("\nuser panic: {} \n {}", m, l);
    } else {
      println!("\nuser panic: {}", m);
    }
  } else {
    println!("\nuser panic!");
  }
  thread_destroy(0);
  loop {}
}
