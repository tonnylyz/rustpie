pub mod page_table;
pub mod vm_descriptor;

// Workaround for abort symbol not found
#[no_mangle]
pub extern "C" fn abort() {
  panic!("abort");
}