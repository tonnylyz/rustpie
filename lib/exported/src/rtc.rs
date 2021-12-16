use common::time::RtcTime;
use microcall::message::Message;

pub fn timestamp() -> Result<u64, &'static str> {
  let result = Message::new(
    0, 0, 0, 0,
  ).call(common::server::SERVER_RTC).map_err(|_| "server call failed")?;
  Ok(result.a as u64)
}

pub fn rtc_time64_to_tm(time: u64) -> RtcTime {
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