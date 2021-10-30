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
  if let Some(m) = info.message() {
    if let Some(l) = info.location() {
      println!("          [E][trusted] panic t{} {} {}", get_tid(), m, l);
    } else {
      println!("          [E][trusted] panic t{} {}", get_tid(), m);
    }
  } else {
    println!("          [E][trusted] panic t{} no message", get_tid());
  }
  unwind::unwind_from_panic(1);
}

#[lang = "eh_personality"]
#[no_mangle]
pub extern fn rust_eh_personality() {
  println!("rust_eh_personality called");
  loop {}
}

#[allow(non_snake_case)]
#[no_mangle]
extern "C" fn _Unwind_Resume(arg: usize) -> ! {
  unwind::unwind_resume(arg)
}
