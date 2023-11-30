use core::arch::x86_64::_rdtsc;

pub fn next() {
  // the APIC timer is set as interval mode
  // nothing to do here
}

// static FREQUENCY: Once<usize> = Once::new();
pub fn frequency() -> usize {
  // NOTE: TSC frequency doesn't have a common way to obtain. As we only support QEMU for now, guess a frequency here.
  // *FREQUENCY.call_once(|| {
  //   let cpuid = raw_cpuid::CpuId::new();
  //   // let freq_info = cpuid.get_processor_frequency_info().unwrap();
  //   // let mhz = freq_info.processor_base_frequency() as usize; // base freq in MHz
  //   // let tsc_info = cpuid.get_tsc_info().unwrap();
  //   // let mhz = tsc_info.tsc_frequency().unwrap() as usize; // tsc freq in MHz
  //   // mhz * 1000 * 1000
  // })
  5_000_000_000 // TODO: proper method to get TSC frequency
}

pub fn counter() -> usize {
  unsafe { _rdtsc() as usize }
}

pub fn init() {
  next();
}
