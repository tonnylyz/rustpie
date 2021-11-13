pub use plic::{Interrupt, INTERRUPT_CONTROLLER};

#[cfg(feature = "k210")]
#[path = "uart_k210.rs"]
pub mod uart;

#[cfg(not(feature = "k210"))]
#[path = "uart_ns16550.rs"]
pub mod uart;

pub mod plic;
mod sbi;
pub mod hsm;
pub mod timer;

