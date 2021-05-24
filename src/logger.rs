use log::{Record, Level, Metadata};
use log::{SetLoggerError, LevelFilter};
use spin::Mutex;

struct SimpleLogger;

static LOCK: Mutex<()> = Mutex::new(());

impl log::Log for SimpleLogger {
  fn enabled(&self, metadata: &Metadata) -> bool {
    metadata.level() <= Level::Info
  }

  fn log(&self, record: &Record) {
    let lock = LOCK.lock();
    if self.enabled(record.metadata()) {
      match record.level() {
        Level::Error =>
          print!("[E]"),
        Level::Warn =>
          print!("[W]"),
        Level::Info =>
          print!("[I]"),
        Level::Debug =>
          print!("[D]"),
        Level::Trace =>
          print!("[T]"),
      }
      if let Some(m) = record.module_path() {
        print!("[{}]", m);
      }
      print!(" {}", record.args());
      println!();
    }
    drop(lock);
  }

  fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init() -> Result<(), SetLoggerError> {
  log::set_logger(&LOGGER)
    .map(|()| log::set_max_level(LevelFilter::Trace))
}
