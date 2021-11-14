use core::ops::{Index, IndexMut};

use gimli::Register;

#[macro_export]
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

#[derive(Clone)]
pub struct Registers {
  registers: [Option<u64>; 32],
}

impl Default for Registers {
  fn default() -> Self {
    Registers {
      registers: [None; 32]
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
