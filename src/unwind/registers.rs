use core::ops::{Index, IndexMut};
use gimli::Register;
use core::fmt::{Debug, Formatter};

#[derive(Clone)]
pub struct Registers {
  registers: [Option<u64>; 96],
}

impl Debug for Registers {
  fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
    for (i, reg) in self.registers.iter().enumerate() {
      match *reg {
        None => { } // write!(f, "[{}]: None, ", i)?,
        Some(r) => write!(f, "[{}]: {:#X}, ", i, r)?,
      }
    }
    Ok(())
  }
}

impl Default for Registers {
  fn default() -> Self {
    Registers {
      registers: [None; 96]
    }
  }
}

impl Index<gimli::Register> for Registers {
  type Output = Option<u64>;
  fn index(&self, index: Register) -> &Self::Output {
    &self.registers[index.0 as usize]
  }
}

impl IndexMut<gimli::Register> for Registers {
  fn index_mut(&mut self, index: Register) -> &mut Self::Output {
    &mut self.registers[index.0 as usize]
  }
}

#[repr(C)]
pub struct LandingRegisters {
  pub r: [u64; 29], // x0-x28
  pub fp: u64,      // x29, Frame Pointer
  pub lr: u64,      // x30, Link Register
  pub sp: u64,      // x31, Stack Pointer
  //vector_half: [u64; 32], // d0-d31
}

// callee-saved
#[repr(C)]
pub struct SavedRegs {
  pub r: [u64; 11], // x19-x29
  pub lr: u64,
  //vector_half: [u64; 8], // d8-d15
}