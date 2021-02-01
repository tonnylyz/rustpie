use super::sbi::{sbi_call, SBI_EID_HSM, SBI_FID_HART_START, SBI_FID_HART_STOP};
use crate::driver::riscv64_virt::sbi::{Error, SBI_FID_HART_GET_STATUS};

pub enum HartStatus {
  Started = 0,
  Stopped = 1,
  StartPending = 2,
  StopPending = 3,
  Unknown,
}

impl From<isize> for HartStatus {
  fn from(i: isize) -> Self {
    match i {
      0 => HartStatus::Started,
      1 => HartStatus::Stopped,
      2 => HartStatus::StartPending,
      3 => HartStatus::StopPending,
      _ => HartStatus::Unknown,
    }
  }
}

pub fn hart_start(hartid: usize, start_addr: usize, opaque: usize) -> Result<(), super::sbi::Error> {
  let r = sbi_call(SBI_EID_HSM, SBI_FID_HART_START, hartid, start_addr, opaque);
  match r {
    Ok(_) => {Ok(())}
    Err(e) => {
      Err(e)
    }
  }
}

pub fn hart_stop_self() -> Result<(), super::sbi::Error> {
  let r = sbi_call(SBI_EID_HSM, SBI_FID_HART_STOP, 0, 0, 0);
  match r {
    Ok(_) => {Ok(())}
    Err(e) => {
      Err(e)
    }
  }
}

pub fn hart_get_status(hartid: usize) -> Result<HartStatus, super::sbi::Error> {
  let r = sbi_call(SBI_EID_HSM, SBI_FID_HART_GET_STATUS, hartid, 0, 0);
  match r {
    Ok(i) => {Ok(i.into())}
    Err(e) => {
      Err(e)
    }
  }
}
