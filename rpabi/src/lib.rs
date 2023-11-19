#![no_std]

cfg_if::cfg_if! {
  if #[cfg(target_arch = "aarch64")] {
    pub const CONFIG_RECURSIVE_PAGE_TABLE_BTM: usize = 0x3f_c000_0000;
  } else if #[cfg(target_arch = "riscv64")] {
    pub const CONFIG_RECURSIVE_PAGE_TABLE_BTM: usize = 0; // Note: not used
    pub const CONFIG_READ_ONLY_LEVEL_3_PAGE_TABLE_BTM: usize = 0x3f_c000_0000;
    pub const CONFIG_READ_ONLY_LEVEL_2_PAGE_TABLE_BTM: usize = 0x3f_c000_0000 - 0x20_0000;
    pub const CONFIG_READ_ONLY_LEVEL_1_PAGE_TABLE_BTM: usize = 0x3f_c000_0000 - 0x20_0000 - 0x1000; // 4 KB
  } else if #[cfg(target_arch = "x86_64")] { 
    pub const CONFIG_RECURSIVE_PAGE_TABLE_BTM: usize = 0x7F80_0000_0000; // 4 level page table
  }else {
    compile_error!("unsupported target_arch");
  }
}

// functions in `string.h`
pub mod string;

pub const CONFIG_USER_LIMIT: usize = 0x3f_a000_0000;
pub const CONFIG_USER_STACK_TOP: usize = 0x3f_8000_0000;
pub const CONFIG_USER_STACK_BTM: usize = 0x3f_0000_0000;
pub const CONFIG_EXCEPTION_STACK_TOP: usize = 0x3f_8000_1000;
pub const CONFIG_HEAP_BTM: usize = 0x10_0000_0000;
pub const CONFIG_VIRTUAL_HEAP_BTM: usize = 0x20_0000_0000;
pub const CONFIG_VIRTUAL_HEAP_TOP: usize = 0x20_1000_0000;

pub const CONFIG_TRUSTED_PLATFORM_INFO: usize = 0x4000_0000;

pub const CONFIG_ELF_IMAGE: usize = 0x8000_0000;

pub const PAGE_SIZE: usize = 4096;

pub const PAGE_TABLE_L1_SHIFT: usize = 30;
pub const PAGE_TABLE_L2_SHIFT: usize = 21;
pub const PAGE_TABLE_L3_SHIFT: usize = 12;

pub const WORD_SHIFT: usize = 3;
pub const WORD_SIZE: usize = 1 << WORD_SHIFT;

pub mod platform {
  pub const DEVICE_NAME_LEN: usize = 32;
  pub const PLATFORM_DEVICE_LEN: usize = 8;
  pub const USER_SPACE_DRIVER_MMIO_OFFSET: usize = 0x8_0000_0000;
  #[derive(Debug, PartialEq, Eq, Clone, Copy)]
  pub enum Driver {
    VirtioBlk,
    Ns16550,
    Pl011,
    Pl031,
    GoldfishRtc,
  }
  #[derive(Debug)]
  pub struct Device {
    pub name: [u8; DEVICE_NAME_LEN],
    pub register: core::ops::Range<usize>,
    pub interrupt: Option<usize>,
    pub driver: Option<Driver>,
  }

  #[repr(align(4096))]
  #[derive(Debug, Default)]
  pub struct PlatformInfo {
    pub devices: [Option<Device>; PLATFORM_DEVICE_LEN],
  }
}

pub mod syscall {
  pub const SYS_NULL: usize = 0;
  pub const SYS_PUTC: usize = 1;
  pub const SYS_GET_ASID: usize = 2;
  pub const SYS_GET_TID: usize = 3;
  pub const SYS_THREAD_YIELD: usize = 4;
  pub const SYS_THREAD_DESTROY: usize = 5;
  pub const SYS_EVENT_WAIT: usize = 6;
  pub const SYS_MEM_ALLOC: usize = 7;
  pub const SYS_MEM_MAP: usize = 8;
  pub const SYS_MEM_UNMAP: usize = 9;
  pub const SYS_ADDRESS_SPACE_ALLOC: usize = 10;
  pub const SYS_THREAD_ALLOC: usize = 11;
  pub const SYS_THREAD_SET_STATUS: usize = 12;
  pub const SYS_ADDRESS_SPACE_DESTROY: usize = 13;
  pub const SYS_ITC_RECV: usize = 14;
  pub const SYS_ITC_SEND: usize = 15;
  pub const SYS_ITC_CALL: usize = 16;
  pub const SYS_SERVER_REGISTER: usize = 17;
  pub const SYS_SERVER_TID: usize = 18;
  pub const SYS_SET_EXCEPTION_HANDLER: usize = 19;
  pub const SYS_GETC: usize = 20;
  pub const SYS_YIELD_TO: usize = 21;
  pub const SYS_REPLY_RECV: usize = 22;
  pub const SYS_MAX: usize = 23;

  pub mod error {
    pub const ERROR_INVARG: usize = 1;
    pub const ERROR_OOM: usize = 2;
    pub const ERROR_MEM_NOT_MAP: usize = 3;
    pub const ERROR_INTERNAL: usize = 4;
    pub const ERROR_DENIED: usize = 5;
    pub const ERROR_HOLD_ON: usize = 6;
    pub const ERROR_OOR: usize = 7;
    pub const ERROR_PANIC: usize = 8;
  }
}

pub mod server {
  pub const SERVER_BLK: usize = 0;
  pub const SERVER_REDOX_FS: usize = 1;
  pub const SERVER_TERMINAL: usize = 2;
  pub const SERVER_MM: usize = 3;
  pub const SERVER_PM: usize = 4;
  pub const SERVER_RTC: usize = 5;
  pub const SERVER_TEST: usize = 6;
}

pub mod thread {
  pub const THREAD_STATUS_RUNNABLE: usize = 1;
  pub const THREAD_STATUS_NOT_RUNNABLE: usize = 2;
}

pub mod event {
  pub const EVENT_INTERRUPT: usize = 1;
  pub const EVENT_THREAD_EXIT: usize = 2;
}

pub mod time {
  use core::fmt::{Display, Formatter};

  /// same as `struct rtc_time` in linux kernel
  #[derive(Default)]
  pub struct RtcTime {
    pub sec: i32,
    pub min: i32,
    pub hour: i32,
    pub mday: i32,
    pub mon: i32,
    pub year: i32,
    pub wday: i32,
    pub yday: i32,
    pub isdst: i32,
  }

  impl Display for RtcTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
      write!(
        f,
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        self.year + 1900,
        self.mon + 1,
        self.mday,
        self.hour,
        self.min,
        self.sec
      )
    }
  }

  impl RtcTime {
    pub fn from_timestamp(time: u64) -> Self {
      let leaps_thru_end_of = |y: i32| (y) / 4 - (y) / 100 + (y) / 400;
      let is_leap_year = |y: i32| ((y % 4 == 0) && (y % 100 != 0)) || (y % 400 == 0);
      let rtc_month_days = |month: i32, year: i32| -> i32 {
        const RTC_DAYS_IN_MONTH: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        RTC_DAYS_IN_MONTH[month as usize] as i32
          + if is_leap_year(year) && month == 1 {
            1
          } else {
            0
          }
      };
      let mut r = RtcTime::default();
      let mut days: i32 = (time / 86400) as i32;
      let mut secs: i32 = (time - (days as u64) * 86400) as i32;
      r.wday = ((days + 4) % 7) as i32;
      let mut year = 1970 + days / 365;
      days -= (year - 1970) * 365 + leaps_thru_end_of(year - 1) - leaps_thru_end_of(1970 - 1);
      if days < 0 {
        year -= 1;
        days += 365 + if is_leap_year(year) { 1 } else { 0 };
      }
      r.year = (year - 1900) as i32;
      r.yday = (days + 1) as i32;
      let mut month = 0;
      loop {
        if month == 12 {
          break;
        }
        let newdays = days - rtc_month_days(month, year);
        if newdays < 0 {
          break;
        }
        days = newdays;
        month += 1;
      }
      r.mon = month as i32;
      r.mday = (days + 1) as i32;
      r.hour = (secs / 3600) as i32;
      secs -= r.hour * 3600;
      r.min = (secs / 60) as i32;
      r.sec = (secs - r.min * 60) as i32;
      r.isdst = 0;
      r
    }
  }
}


#[derive(Debug)]
pub struct X64BootData {
  pub free_mem_start: usize,
  pub free_mem_count: usize,
}
