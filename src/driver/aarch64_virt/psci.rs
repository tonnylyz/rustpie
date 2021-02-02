use super::smc::*;

enum Function {
  //Version = 0x84000000,
  //SuspendAarch64 = 0xc4000001,
  //CpuOff = 0x84000002,
  CpuOnAarch64 = 0xc4000003,
  //AffinityInfoAarch64 = 0xc4000004,
  //Features = 0x8400000A,
  //MigInfoType = 0x84000006,
  //SystemOff = 0x84000008,
  //SystemReset = 0x84000009,
}

// enum Error {
//   Success = 0,
//   NotSupported = -1,
//   InvalidParams = -2,
//   Denied = -3,
//   AlreadyOn = -4,
//   OnPending = -5,
//   InternalFailure = -6,
//   NotPresent = -7,
//   Disabled = -8,
//   InvalidAddress = -9,
// }

pub fn cpu_on(mpidr: u64, entry: u64, x0: u64) {
  let _ = smc_call(Function::CpuOnAarch64 as u64, mpidr, entry, x0);
}
