pub use gic::{Interrupt, INTERRUPT_CONTROLLER};

#[cfg(not(feature = "gicv3"))]
#[path ="gic.rs"]
pub mod gic;

#[cfg(feature = "gicv3")]
#[path ="gicv3.rs"]
pub mod gic;

mod smc;
pub mod psci;
pub mod timer;

