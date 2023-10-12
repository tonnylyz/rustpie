use rpabi::time::RtcTime;

use rpsyscall::get_tid;

#[cfg(target_arch = "aarch64")]
#[cfg(feature = "virt")]
pub fn timestamp() -> u64 {
  const PL031_MMIO_BASE: usize = 0x8_0000_0000 + 0x9010000;
  unsafe { (PL031_MMIO_BASE as *mut u32).read() as u64 }
}

#[cfg(target_arch = "riscv64")]
#[cfg(feature = "virt")]
pub fn timestamp() -> u64 {
  const NSEC_PER_SEC: u64 = 1000000000;
  const GOLDFISH_MMIO_BASE: usize = 0x8_0000_0000 + 0x101000;
  let low = unsafe { (GOLDFISH_MMIO_BASE as *mut u32).read() as u64 };
  let high = unsafe { ((GOLDFISH_MMIO_BASE + 4) as *mut u32).read() as u64 };
  ((high << 32) | low) / NSEC_PER_SEC
}

#[cfg(target_arch = "riscv64")]
#[cfg(feature = "k210")]
pub fn timestamp() -> u64 { 0 }

fn rtc_time64_to_tm(time: u64) -> RtcTime {
  let leaps_thru_end_of = |y: i32| {
    (y) / 4 - (y) / 100 + (y) / 400
  };
  let is_leap_year = |y: i32| {
    ((y % 4 == 0) && (y % 100 != 0)) || (y % 400 == 0)
  };
  let rtc_month_days = |month: i32, year: i32| -> i32 {
    const RTC_DAYS_IN_MONTH: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    RTC_DAYS_IN_MONTH[month as usize] as i32 + if is_leap_year(year) && month == 1 { 1 } else { 0 }
  };
  let mut r = RtcTime::default();
  let mut days: i32 = (time / 86400) as i32;
  let mut secs: i32 = (time - (days as u64) * 86400) as i32;
  r.wday = ((days + 4) % 7) as i32;
  let mut year = 1970 + days / 365;
  days -= (year - 1970) * 365
    + leaps_thru_end_of(year - 1)
    - leaps_thru_end_of(1970 - 1);
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

#[allow(dead_code)]
pub fn server() {
  info!("server started t{}",  get_tid());
  rpsyscall::server_register(rpabi::server::SERVER_RTC).unwrap();
  info!("start at {} {}", timestamp(), rtc_time64_to_tm(timestamp() as u64));
  loop {
    let (client_tid, _msg) = rpsyscall::message::Message::receive().unwrap();
    let mut msg = rpsyscall::message::Message::default();
    msg.a = timestamp() as usize;
    let _ = msg.send_to(client_tid);
  }
}
