pub use gic::{Interrupt, INTERRUPT_CONTROLLER};

pub mod gic;
mod smc;
pub mod psci;
pub mod timer;

