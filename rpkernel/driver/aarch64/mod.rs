pub use gic::{Interrupt, INTERRUPT_CONTROLLER};

#[cfg(feature = "virt")]
pub mod uart;


pub mod gic;
mod smc;
pub mod psci;
pub mod timer;

