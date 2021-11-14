pub const SBI_EID_TIMER: u32 = 0x54494D45;
pub const SBI_FID_SET_TIMER: u32 = 0x00;

pub const SBI_EID_HSM: u32 = 0x48534D;
pub const SBI_FID_HART_START: u32 = 0x00;
//pub const SBI_FID_HART_STOP: u32 = 0x01;
//pub const SBI_FID_HART_GET_STATUS: u32 = 0x02;

pub enum Error {
  Failed = 1,
  NotSupported = 2,
  InvalidParam = 3,
  Denied = 4,
  InvalidAddress = 5,
  AlreadyAvailable = 6,
  Unknown,
}

impl From<isize> for Error {
  fn from(i: isize) -> Self {
    match i {
      -1 => Error::Failed,
      -2 => Error::NotSupported,
      -3 => Error::InvalidParam,
      -4 => Error::Denied,
      -5 => Error::InvalidAddress,
      -6 => Error::AlreadyAvailable,
      _ => Error::Unknown,
    }
  }
}

#[repr(C)]
struct SbiReturn {
  error: isize,
  value: isize,
}

#[inline(always)]
pub fn sbi_call(eid: u32, fid: u32, arg0: usize, arg1: usize, arg2: usize) -> Result<isize, Error> {
  let mut ret: SbiReturn = SbiReturn { error: 0, value: 0 };
  unsafe {
    asm!("ecall",
    inlateout("x10") arg0 => ret.error,
    inlateout("x11") arg1 => ret.value,
    in("x12") arg2, in("x16") fid, in("x17") eid);
  }
  if ret.error == 0 {
    Ok(ret.value)
  } else {
    Err(ret.error.into())
  }
}
