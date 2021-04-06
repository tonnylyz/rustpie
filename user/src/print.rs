use core::fmt;

use crate::syscall::{putc, thread_destroy};

struct Writer;

static mut WRITER: Writer = Writer;

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
  unsafe {
    WRITER.write_fmt(args).unwrap();
  }
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
  match thread_destroy(0) {
    Ok(_) => {}
    Err(_) => { println!("user: panic_handler: process_destroy failed"); }
  }
  loop {}
}