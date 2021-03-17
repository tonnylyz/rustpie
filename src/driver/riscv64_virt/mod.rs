pub mod uart;
#[allow(dead_code)]
pub mod plic;

mod sbi;
pub mod hsm;
pub mod timer;

pub use plic::INTERRUPT_CONTROLLER;