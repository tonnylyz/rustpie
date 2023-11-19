use x86_64::{set_general_handler, VirtAddr};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use super::ContextFrame;

pub static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub fn init() {
  extern "C" {
      fn syscall_entry();
  }
  unsafe {
    let idt = &mut IDT;
    set_general_handler!(idt, abort, 0..32);
    set_general_handler!(idt, unhandle, 32..64);
    set_general_handler!(idt, unknown, 64..);
    idt.stack_segment_fault.set_handler_fn(stack_segment_fault);
    idt.general_protection_fault.set_handler_fn(general_protection_fault);
    idt.page_fault.set_handler_fn(page_fault_handler);
    IDT.load();
    x86_64::registers::model_specific::LStar::write(VirtAddr::new(syscall_entry as u64));
  }
}

fn abort(stack_frame: InterruptStackFrame, index: u8, error_code: Option<u64>) {
  println!("abort");
  loop {}
}
fn unhandle(stack_frame: InterruptStackFrame, index: u8, error_code: Option<u64>) {
  println!("unhandle");
  loop {}
}
fn unknown(stack_frame: InterruptStackFrame, index: u8, error_code: Option<u64>) {
  println!("unknown");
  loop {}
}

core::arch::global_asm!(include_str!("interrupt.S"));

extern "x86-interrupt" fn page_fault_handler(
  stack_frame: InterruptStackFrame,
  error_code: x86_64::structures::idt::PageFaultErrorCode,
) {
  println!("page fault");
  println!(
    "fault address: {:?}",
    x86_64::registers::control::Cr2::read()
  );
  println!("error code: {:?}", error_code);
  loop {}
}

extern "x86-interrupt" fn general_protection_fault(
  stack_frame: InterruptStackFrame,
  error_code: u64,
) {
  println!("general protection fault");
  println!("error code: {:?}", error_code);
  loop {}
}

extern "x86-interrupt" fn stack_segment_fault(
  stack_frame: InterruptStackFrame,
  error_code: u64,
) {
  println!("stack_segment_fault");
  println!("error code: {:?}", error_code);
  loop {}
}

#[no_mangle]
extern "C" fn syscall_rust_entry(ctx: *mut ContextFrame) {
  let core = crate::kernel::cpu::cpu();
  core.set_context(ctx);
  crate::kernel::syscall::syscall();
  core.clear_context();
}
