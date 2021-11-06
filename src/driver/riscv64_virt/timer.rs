use riscv::regs::*;
use tock_registers::interfaces::{Readable, Writeable};

use super::sbi::{sbi_call, SBI_EID_TIMER, SBI_FID_SET_TIMER};

const TIMER_DEFAULT_COUNT: usize = 250000;

pub fn next() {
  let _ = sbi_call(SBI_EID_TIMER, SBI_FID_SET_TIMER, counter() + TIMER_DEFAULT_COUNT, 0, 0);
}

// NOTE: timer frequency can be obtained from FDT
// 	cpus {
// 		#address-cells = <0x01>;
// 		#size-cells = <0x00>;
// 		timebase-frequency = <0x989680>;
#[cfg(not(feature = "k210"))]
const TIMER_FREQUENCY: usize = 0x989680;

#[cfg(feature = "k210")]
const TIMER_FREQUENCY: usize = 7800000;

pub fn frequency() -> usize {
  TIMER_FREQUENCY
}

pub fn counter() -> usize {
  TIME.get() as usize
}

pub fn init() {
  next();
  SIE.write(SIE::STIE::SET);
}
