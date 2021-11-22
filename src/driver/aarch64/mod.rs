pub use gic::{Interrupt, INTERRUPT_CONTROLLER};

#[cfg(not(feature = "tx2"))]
pub mod uart;

#[cfg(feature = "tx2")]
#[path = "uart_ns16550.rs"]
pub mod uart;


pub mod gic;
mod smc;
pub mod psci;
pub mod timer;

