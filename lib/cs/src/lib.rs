#![no_std]

pub mod mm {
  pub mod action {
    pub const ALLOC: usize = 1;
  }

  pub mod result {
    pub const OK: usize = 0;
    pub const ERR: usize = 1;
    pub const UNKNOWN_ACTION: usize = 2;
  }
}

pub mod pm {
  pub mod action {
    pub const SPAWN: usize = 1;
    pub const WAIT: usize = 2;
    pub const PS: usize = 3;
  }

  pub mod result {
    pub const OK: usize = 0;
    pub const HOLD_ON: usize = 1;
    pub const INVARG: usize = 2;
    pub const SPAWN_FAILED: usize = 3;
  }
}

pub mod blk {
  pub mod action {
    pub const READ: usize = 0;
    pub const WRITE: usize = 1;
    pub const SIZE: usize = 2;
  }

  pub mod result {
    pub const OK: usize = 0;
    pub const ERR: usize = 1;
  }
}
