use core::panic::PanicInfo;

use crate::lib::traits::*;

#[derive(Debug, Clone)]
struct Frame {
  fp: u64,
  pc: u64,
}

impl Frame {
  fn new(fp: u64, pc: u64) -> Frame {
    Frame {
      fp,
      pc,
    }
  }

  #[cfg(target_arch = "aarch64")]
  fn current() -> Frame {
    let fp: u64;
    let pc: u64;
    unsafe {
      core::arch::asm!("mov {}, x29", out(reg) fp);
      core::arch::asm!("adr {}, #0", out(reg) pc);
    }
    Frame::new(fp, pc)
  }

  #[cfg(target_arch = "riscv64")]
  fn current() -> Frame {
    let fp: u64;
    let pc: u64;
    unsafe {
      core::arch::asm!("mv {}, x8", out(reg) fp);
      core::arch::asm!("auipc {}, 0", out(reg) pc);
    }
    Frame::new(fp, pc)
  }

  fn ip(&self) -> *mut u8 {
    self.pc as *mut u8
  }
}

#[inline(always)]
fn trace_from(mut curframe: Frame, cb: &mut dyn FnMut(&Frame) -> bool) {
  loop {
    let ctxt = curframe.clone();

    let keep_going = cb(&ctxt);

    if keep_going {
      unsafe {
        #[cfg(target_arch = "aarch64")]
          {
            curframe.pc = ((curframe.fp + 8) as *mut u64).read();
            curframe.fp = (curframe.fp as *mut u64).read();
          }
        #[cfg(target_arch = "riscv64")]
          {
            curframe.pc = ((curframe.fp - 8) as *mut u64).read();
            curframe.fp = ((curframe.fp - 16) as *mut u64).read();
          }

        if curframe.pc == 0 || curframe.fp <= 0xfff {
          break;
        }
      }
    } else {
      break;
    }
  }
}

#[inline(always)]
fn trace(cb: &mut dyn FnMut(&Frame) -> bool) {
  trace_from(Frame::current(), cb);
}

#[allow(dead_code)]
pub fn exception_trace() {
  let ctx = crate::lib::cpu::cpu().context();
  #[cfg(target_arch = "aarch64")]
    let frame_zero = Frame {
    pc: ctx.exception_pc() as u64,
    fp: ctx.gpr(29) as u64,
  };

  #[cfg(target_arch = "riscv64")]
    let frame_zero = Frame {
    pc: ctx.exception_pc() as u64,
    fp: ctx.gpr(8) as u64,
  };

  backtrace_from(frame_zero);
}

#[inline(always)]
fn backtrace_from(frame_zero: Frame) {
  info!("backtrace begin");
  let mut count = 0;
  trace_from(frame_zero, &mut |frame: &Frame| {
    count += 1;
    let ip = frame.ip();
    info!("frame #{:02} - {:016x}", count, ip as usize);
    true
  });
}

#[allow(dead_code)]
fn backtrace() {
  info!("backtrace begin");
  let mut count = 0;
  trace(&mut |frame: &Frame| {
    count += 1;
    let ip = frame.ip();
    info!("frame #{:02} - {:016x}", count, ip as usize);
    true
  });
}

#[panic_handler]
pub fn panic_handler(info: &PanicInfo) -> ! {
  if let Some(message) = info.message() {
    error!("PANIC: {}", message);
  }
  if let Some(location) = info.location() {
    error!("Location: {}:{}", location.file(), location.line());
  }

  // backtrace();
  // info!("backtrace done");

  unwind::unwind_from_panic(1);
}

#[allow(dead_code)]
static mut PANICKED: bool = false;

#[allow(dead_code)]
pub fn random_panic() {
  unsafe {
    if !PANICKED {
      PANICKED = true;
      panic!("[[RANDOM]][[PANIC]]");
    }
  }
}

#[allow(dead_code)]
static mut PAGEFAULT: bool = false;

#[allow(dead_code)]
pub fn random_page_fault() {
  unsafe {
    if !PAGEFAULT {
      PAGEFAULT = true;
      info!("[[RANDOM]][[PAGEFAULT]]");
      (0xdeafbeef00000000 as *mut u32).read_volatile();
    }
  }
}
