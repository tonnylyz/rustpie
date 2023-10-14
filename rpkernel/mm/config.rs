// non paged memory in kernel (kernel heap memory)
#[cfg(not(feature = "k210"))]
pub const CONFIG_NON_PAGED_MEMORY_SIZE: usize = 0xf00_0000;

#[cfg(feature = "k210")]
pub const CONFIG_NON_PAGED_MEMORY_SIZE: usize = 0x10_0000;
