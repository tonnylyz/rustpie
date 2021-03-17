pub mod uart;
pub mod timer;
mod smc;
pub mod psci;
pub mod gic;

pub use gic::INTERRUPT_CONTROLLER;