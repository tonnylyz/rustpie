use core::ops::{Index, IndexMut};
use gimli::Register;
use core::fmt::{Debug, Formatter};

macro_rules! registers {
    ($struct_name:ident, { $($name:ident = ($val:expr, $disp:expr)),+ $(,)? }) => {
        #[allow(missing_docs)]
        impl $struct_name {
            $(
                pub const $name: Register = Register($val);
            )+
        }

        impl $struct_name {
            /// The name of a register, or `None` if the register number is unknown.
            #[allow(dead_code)]
            pub fn register_name(register: Register) -> Option<&'static str> {
                match register {
                    $(
                        Self::$name => Some($disp),
                    )+
                    _ => return None,
                }
            }
        }
    };
}

/// Aarch64 architecture specific definitions.
#[derive(Debug, Clone, Copy)]
pub struct Aarch64;

registers!(Aarch64, {
    X0 = (0, "X0"),
    X1 = (1, "X1"),
    X2 = (2, "X2"),
    X3 = (3, "X3"),
    X4 = (4, "X4"),
    X5 = (5, "X5"),
    X6 = (6, "X6"),
    X7 = (7, "X7"),
    X8 = (8, "X8"),
    X9 = (9, "X9"),
    X10 = (10, "X10"),
    X11 = (11, "X11"),
    X12 = (12, "X12"),
    X13 = (13, "X13"),
    X14 = (14, "X14"),
    X15 = (15, "X15"),
    X16 = (16, "X16"),
    X17 = (17, "X17"),
    X18 = (18, "X18"),
    X19 = (19, "X19"),
    X20 = (20, "X20"),
    X21 = (21, "X21"),
    X22 = (22, "X22"),
    X23 = (23, "X23"),
    X24 = (24, "X24"),
    X25 = (25, "X25"),
    X26 = (26, "X26"),
    X27 = (27, "X27"),
    X28 = (28, "X28"),
    X29 = (29, "X29"),
    X30 = (30, "X30"),
    SP = (31, "SP"),
});

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
  pub x: [u64; 29], // x0-x28
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