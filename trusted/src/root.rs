use rpabi::platform::PlatformInfo;

use crate::common::thread;
use crate::common::wrapper::server_wrapper;

#[cfg(target_arch = "x86_64")]
pub fn main(_info: &'static PlatformInfo) {
  use crate::common::mm::virt_to_phys;

  info!("trusted main");
  info!("test virt {:X} to phy {:X}", main as usize, virt_to_phys(main as usize));
  loop {}
}

#[cfg(not(target_arch = "x86_64"))]
pub fn main(info: &'static PlatformInfo) {
  let mut join_handlers = vec![];
  let mut has_user_space_serial = false;
  let mut has_user_space_rtc = false;
  for dev in &info.devices {
    match dev {
      Some(dev) => match dev.driver {
        Some(driver) => match driver {
          rpabi::platform::Driver::VirtioBlk => {
            join_handlers.push(thread::spawn(|| {
              crate::blk::virtio_blk::server(dev.register.start, dev.interrupt.unwrap());
            }));
          }
          rpabi::platform::Driver::Ns16550 => {
            has_user_space_serial = true;
            join_handlers.push(thread::spawn(|| {
              crate::serial::ns16550::server(dev.register.start, dev.interrupt.unwrap());
            }));
          }
          rpabi::platform::Driver::Pl011 => {
            has_user_space_serial = true;
            join_handlers.push(thread::spawn(|| {
              crate::serial::pl011::server(dev.register.start, dev.interrupt.unwrap());
            }));
          }
          rpabi::platform::Driver::Pl031 => {
            has_user_space_rtc = true;
            join_handlers.push(thread::spawn(|| {
              crate::rtc::pl031::server(dev.register.start, 0);
            }));
          }
          rpabi::platform::Driver::GoldfishRtc => {
            has_user_space_rtc = true;
            join_handlers.push(thread::spawn(|| {
              crate::rtc::goldfish::server(dev.register.start, 0);
            }));
          }
          rpabi::platform::Driver::Nil => {}
        },
        None => {}
      },
      None => continue,
    }
  }

  if !has_user_space_serial {
    join_handlers.push(thread::spawn(|| {
      server_wrapper(crate::serial::default::input_server);
    }));
    join_handlers.push(thread::spawn(|| {
      server_wrapper(crate::serial::default::server);
    }));
  }

  if !has_user_space_rtc {
    join_handlers.push(thread::spawn(|| {
      server_wrapper(crate::rtc::dummy::server);
    }));
  }

  join_handlers.push(thread::spawn(|| {
    server_wrapper(crate::fs::server::server);
  }));

  join_handlers.push(thread::spawn(|| {
    server_wrapper(crate::mm::server);
  }));

  join_handlers.push(thread::spawn(|| {
    server_wrapper(crate::pm::server);
  }));

  join_handlers.push(thread::spawn(|| {
    match crate::common::loader::spawn("shell") {
      Ok((_asid, tid)) => {
        rpsyscall::thread_set_status(tid, rpabi::thread::THREAD_STATUS_RUNNABLE)
          .expect("root start shell failed");
      }
      Err(s) => {
        error!("{}", s);
      }
    }
  }));
  for handler in join_handlers {
    handler.join().expect("root join thread failed");
  }
}
