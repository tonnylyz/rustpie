pub mod uart;
pub mod gic;
mod smc;
pub mod psci;
pub mod timer;

pub use gic::{INTERRUPT_CONTROLLER, Interrupt};