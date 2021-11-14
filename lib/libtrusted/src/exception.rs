#[cfg(target_arch = "aarch64")]
use unwind::arch::Aarch64;
#[cfg(target_arch = "riscv64")]
use unwind::arch::Riscv64;
use unwind::registers::Registers;
use unwind::unwind_from_exception;

#[cfg(target_arch = "aarch64")]
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ContextFrame {
  gpr: [u64; 31],
  spsr: u64,
  elr: u64,
  sp: u64,
}

#[cfg(target_arch = "riscv64")]
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ContextFrame {
  gpr: [u64; 32],
  sstatus: u64,
  sepc: u64,
}

#[cfg(target_arch = "riscv64")]
impl Into<Registers> for ContextFrame {
  fn into(self) -> Registers {
    let mut reg = Registers::default();
    reg[Riscv64::X0] = Some(self.gpr[0]);
    reg[Riscv64::X1] = Some(self.gpr[1]);
    reg[Riscv64::X2] = Some(self.gpr[2]);
    reg[Riscv64::X3] = Some(self.gpr[3]);
    reg[Riscv64::X4] = Some(self.gpr[4]);
    reg[Riscv64::X5] = Some(self.gpr[5]);
    reg[Riscv64::X6] = Some(self.gpr[6]);
    reg[Riscv64::X7] = Some(self.gpr[7]);
    reg[Riscv64::X8] = Some(self.gpr[8]);
    reg[Riscv64::X9] = Some(self.gpr[9]);
    reg[Riscv64::X10] = Some(self.gpr[10]);
    reg[Riscv64::X11] = Some(self.gpr[11]);
    reg[Riscv64::X12] = Some(self.gpr[12]);
    reg[Riscv64::X13] = Some(self.gpr[13]);
    reg[Riscv64::X14] = Some(self.gpr[14]);
    reg[Riscv64::X15] = Some(self.gpr[15]);
    reg[Riscv64::X16] = Some(self.gpr[16]);
    reg[Riscv64::X17] = Some(self.gpr[17]);
    reg[Riscv64::X18] = Some(self.gpr[18]);
    reg[Riscv64::X19] = Some(self.gpr[19]);
    reg[Riscv64::X20] = Some(self.gpr[20]);
    reg[Riscv64::X21] = Some(self.gpr[21]);
    reg[Riscv64::X22] = Some(self.gpr[22]);
    reg[Riscv64::X23] = Some(self.gpr[23]);
    reg[Riscv64::X24] = Some(self.gpr[24]);
    reg[Riscv64::X25] = Some(self.gpr[25]);
    reg[Riscv64::X26] = Some(self.gpr[26]);
    reg[Riscv64::X27] = Some(self.gpr[27]);
    reg[Riscv64::X28] = Some(self.gpr[28]);
    reg[Riscv64::X29] = Some(self.gpr[29]);
    reg[Riscv64::X30] = Some(self.gpr[30]);
    reg[Riscv64::X31] = Some(self.gpr[31]);
    reg
  }
}

#[cfg(target_arch = "aarch64")]
impl Into<Registers> for ContextFrame {
  fn into(self) -> Registers {
    let mut reg = Registers::default();
    reg[Aarch64::X0] = Some(self.gpr[0]);
    reg[Aarch64::X1] = Some(self.gpr[1]);
    reg[Aarch64::X2] = Some(self.gpr[2]);
    reg[Aarch64::X3] = Some(self.gpr[3]);
    reg[Aarch64::X4] = Some(self.gpr[4]);
    reg[Aarch64::X5] = Some(self.gpr[5]);
    reg[Aarch64::X6] = Some(self.gpr[6]);
    reg[Aarch64::X7] = Some(self.gpr[7]);
    reg[Aarch64::X8] = Some(self.gpr[8]);
    reg[Aarch64::X9] = Some(self.gpr[9]);
    reg[Aarch64::X10] = Some(self.gpr[10]);
    reg[Aarch64::X11] = Some(self.gpr[11]);
    reg[Aarch64::X12] = Some(self.gpr[12]);
    reg[Aarch64::X13] = Some(self.gpr[13]);
    reg[Aarch64::X14] = Some(self.gpr[14]);
    reg[Aarch64::X15] = Some(self.gpr[15]);
    reg[Aarch64::X16] = Some(self.gpr[16]);
    reg[Aarch64::X17] = Some(self.gpr[17]);
    reg[Aarch64::X18] = Some(self.gpr[18]);
    reg[Aarch64::X19] = Some(self.gpr[19]);
    reg[Aarch64::X20] = Some(self.gpr[20]);
    reg[Aarch64::X21] = Some(self.gpr[21]);
    reg[Aarch64::X22] = Some(self.gpr[22]);
    reg[Aarch64::X23] = Some(self.gpr[23]);
    reg[Aarch64::X24] = Some(self.gpr[24]);
    reg[Aarch64::X25] = Some(self.gpr[25]);
    reg[Aarch64::X26] = Some(self.gpr[26]);
    reg[Aarch64::X27] = Some(self.gpr[27]);
    reg[Aarch64::X28] = Some(self.gpr[28]);
    reg[Aarch64::X29] = Some(self.gpr[29]);
    reg[Aarch64::X30] = Some(self.gpr[30]);
    reg[Aarch64::SP] = Some(self.sp);
    reg
  }
}

pub fn handler(ctx: &ContextFrame) {
  info!("exception handler");
  let ctx = ctx.clone();
  let reg = ctx.into();
  unwind_from_exception(reg);
}
