global_asm!(include_str!("page_fault.S"));
global_asm!(include_str!("syscall.S"));

pub mod page_table;
pub mod vm_descriptor;