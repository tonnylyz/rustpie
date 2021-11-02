use std::time::Instant;
use crate::verbose;

pub const ITERATIONS: usize = 10_000;
pub const TRIES: usize = 10;
pub const THRESHOLD_ERROR_RATIO: u64 = 2;
pub const _SEC_TO_NANO: f64 = 1_000_000_000.0;

macro_rules! printlninfo {
    ($fmt:expr) => (if verbose() {println!(concat!("BM-INFO: ", $fmt))});
    ($fmt:expr, $($arg:tt)*) => (if verbose() {println!(concat!("BM-INFO: ", $fmt), $($arg)*)});
}

macro_rules! printlnwarn {
    ($fmt:expr) => (if verbose() {println!(concat!("BM-WARN: ", $fmt))});
    ($fmt:expr, $($arg:tt)*) => (if verbose() {println!(concat!("BM-WARN: ", $fmt), $($arg)*)});
}

// overhead is TIME
pub fn timing_overhead_inner(th: usize, nr: usize) -> u64 {
  let mut temp;
  let start;
  let end;

  temp = Instant::now();

  start = Instant::now();
  for _ in 0..ITERATIONS {
    temp = Instant::now();
  }
  end = Instant::now();

  let delta = end - start;
  let delta_time = delta.as_nanos() as u64;
  let delta_time_avg = delta_time / ITERATIONS as u64;

  printlninfo!("t_overhead_inner ({}/{}): {} total -> {:.2} avg_ns (ignore: {})",
		th, nr, delta_time, delta_time_avg, temp.elapsed().as_nanos());

  delta_time_avg
}

// overhead is TIME
pub fn timing_overhead() -> u64 {
  let mut tries: u64 = 0;
  let mut max: u64 = core::u64::MIN;
  let mut min: u64 = core::u64::MAX;

  for i in 0..TRIES {
    let overhead = timing_overhead_inner(i+1, TRIES);
    tries += overhead;
    if overhead > max {max = overhead;}
    if overhead < min {min = overhead;}
  }

  let overhead = tries / TRIES as u64;
  // We expect the maximum and minimum to be within 10*THRESHOLD_ERROR_RATIO % of the mean value
  let err = (overhead * 10 * THRESHOLD_ERROR_RATIO) / 100;
  if 	max - overhead > err || overhead - min > err {
    printlnwarn!("timing_overhead diff is too big: {:.2} ({:.2} - {:.2}) ns", max-min, max, min);
  }

  printlninfo!("Timing overhead: {} ns\n\n", overhead);

  overhead
}