use spin::Mutex;

struct Writer;

static WRITER: Mutex<Writer> = Mutex::new(Writer);

impl core::fmt::Write for Writer {
  fn write_str(&mut self, s: &str) -> core::fmt::Result {
    for c in s.chars() {
      microcall::putc(c);
    }
    Ok(())
  }
}

pub fn print_arg(args: core::fmt::Arguments) {
  use core::fmt::Write;
  let mut writer = WRITER.lock();
  writer.write_fmt(args).unwrap();
}

#[panic_handler]
fn panic_handler(_info: &core::panic::PanicInfo) -> ! {
  loop {}
}
