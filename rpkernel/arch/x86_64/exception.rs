use x86_64::registers::control::{Cr4, Cr4Flags};
use x86_64::registers::model_specific::{
  Efer, EferFlags, GsBase, KernelGsBase, LStar, SFMask, Star,
};
use x86_64::registers::rflags::RFlags;
use x86_64::registers::segmentation::*;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable};
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::{set_general_handler, VirtAddr};

use super::ContextFrame;
use crate::driver::{apic, INTERRUPT_CONTROLLER};
use crate::kernel::interrupt::*;

static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();
static mut TSS: TaskStateSegment = TaskStateSegment::new();

#[repr(C)]
#[derive(Debug)]
struct X64PerCpuData {
  kernel_rsp: usize,
  user_rsp: usize,
  scratch: usize,
}

static mut X64_PER_CPU_DATA: X64PerCpuData = X64PerCpuData {
  kernel_rsp: 0,
  user_rsp: 0,
  scratch: 0,
};

pub static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

pub fn init() {
  extern "C" {
    fn syscall_entry();
  }
  unsafe {
    let kernel_cs = GDT.add_entry(Descriptor::kernel_code_segment()); // 1*8 = 8
    let kernel_ss = GDT.add_entry(Descriptor::kernel_data_segment()); // 2*8 = 16
    let user_cs = GDT.add_entry(Descriptor::user_code_segment()); // 3*8 + 3 = 27
    let user_ss = GDT.add_entry(Descriptor::user_data_segment()); // 4*8 + 3 = 35
    let sysret_cs = GDT.add_entry(Descriptor::user_data_segment()); // 5*8 + 3 = 43

    println!("kernel_cs {:?} {}", kernel_cs, kernel_cs.0);
    println!("kernel_ss {:?} {}", kernel_ss, kernel_ss.0);
    println!("user_cs {:?} {}", user_cs, user_cs.0);
    println!("user_ss {:?} {}", user_ss, user_ss.0);
    println!("sysret_cs {:?} {}", sysret_cs, sysret_cs.0);

    let tss_selector = GDT.add_entry(Descriptor::tss_segment(&TSS));

    Efer::update(|f: &mut EferFlags| {
      f.insert(EferFlags::SYSTEM_CALL_EXTENSIONS);
      f.remove(EferFlags::NO_EXECUTE_ENABLE)
    });
    let efer = Efer::read();
    println!("efer {:?}", efer);
    Cr4::update(|f| f.insert(Cr4Flags::PCID));
    let cr4 = Cr4::read();
    println!("cr4 {:?}", cr4);

    // rustpi uses per-CPU kernel stack (not differentiate for threads)
    // the same stack is use for interrupt, syscall
    let kernel_stack = crate::kernel::stack::stack_of_core(0) as u64;
    
    TSS.privilege_stack_table[0] = VirtAddr::new(kernel_stack); // interrupt kernel stack
    TSS.interrupt_stack_table[0] = VirtAddr::new(kernel_stack); // interrupt kernel stack (when IST(0) is set in IDT)
    X64_PER_CPU_DATA.kernel_rsp = kernel_stack as usize; // syscall kernel stack (retrieved after SWAPGS indexed by GS)

    GDT.load();
    // init all kernel segment selector. Must be done after LGDT
    CS::set_reg(kernel_cs);
    SS::set_reg(kernel_ss);
    DS::set_reg(SegmentSelector(0));
    ES::set_reg(SegmentSelector(0));
    FS::set_reg(SegmentSelector(0));
    GS::set_reg(SegmentSelector(0));
    let per_cpu = VirtAddr::new(&X64_PER_CPU_DATA as *const _ as u64);
    println!("per cpu {:?}", per_cpu);

    // make SWAPGS work here. Supposed we are in kernel, set (GS, KernelGS) to (per_cpu_va, 0). Before entering ring-3, it's swapped back.
    GsBase::write(per_cpu);
    KernelGsBase::write(VirtAddr::new(0));

    x86_64::instructions::tables::load_tss(tss_selector);

    let idt = &mut IDT;
    set_general_handler!(idt, abort, 0..32);
    set_general_handler!(idt, unhandle, 32..64);
    set_general_handler!(idt, unknown, 64..);
    idt
      .stack_segment_fault
      .set_handler_fn(stack_segment_fault)
      .set_stack_index(0);
    idt
      .general_protection_fault
      .set_handler_fn(general_protection_fault)
      .set_stack_index(0);
    idt
      .page_fault
      .set_handler_fn(page_fault_handler)
      .set_stack_index(0);
    // Set timer handler.
    extern "C" {
      // see interrupt.S
      fn timer_interrupt_handler();
    }
    idt[apic::INT_TIMER]
      .set_handler_addr(VirtAddr::new(timer_interrupt_handler as u64))
      .set_stack_index(0);
    idt[apic::ERROR_INTERRUPT_NUMBER as usize]
      .set_handler_fn(error_interrupt_handler)
      .set_stack_index(0);
    idt[apic::SPURIOUS_INTERRUPT_NUMBER as usize]
      .set_handler_fn(spurious_interrupt_handler)
      .set_stack_index(0);
    IDT.load();

    // Syscall
    // set kernel CS/SS when entering kernel by syscall
    // currently rustpi doesn't use sysret to return to user-space
    Star::write(sysret_cs, user_ss, kernel_cs, kernel_ss).unwrap();
    LStar::write(VirtAddr::new(syscall_entry as u64));
    let mut flag_mask = RFlags::empty();
    flag_mask.insert(RFlags::INTERRUPT_FLAG);
    flag_mask.insert(RFlags::IOPL_HIGH);
    flag_mask.insert(RFlags::IOPL_LOW);
    // clear RFLAGS:IF and set IOPL=0 when entering kernel by syscall
    SFMask::write(flag_mask);
  }
}

fn abort(stack_frame: InterruptStackFrame, index: u8, error_code: Option<u64>) {
  println!("abort");
  println!("{:?} index {} error_code {:?}", stack_frame, index, error_code);
  loop {}
}
fn unhandle(stack_frame: InterruptStackFrame, index: u8, error_code: Option<u64>) {
  println!("unhandle");
  println!("{:?} index {} error_code {:?}", stack_frame, index, error_code);
  loop {}
}
fn unknown(stack_frame: InterruptStackFrame, index: u8, error_code: Option<u64>) {
  println!("unknown");
  println!("{:?} index {} error_code {:?}", stack_frame, index, error_code);
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
  println!("error code: {:?} {:?}", error_code, stack_frame);
  loop {}
}

extern "x86-interrupt" fn general_protection_fault(
  stack_frame: InterruptStackFrame,
  error_code: u64,
) {
  println!("general protection fault");
  println!("error code: {:?} {:?}", error_code, stack_frame);
  loop {}
}

extern "x86-interrupt" fn stack_segment_fault(stack_frame: InterruptStackFrame, error_code: u64) {
  println!("stack_segment_fault");
  println!("error code: {:?} {:?}", error_code, stack_frame);
  loop {}
}

#[no_mangle]
extern "C" fn timer_rust_entry(ctx: *mut ContextFrame) {
  let core = crate::kernel::cpu::cpu();
  core.set_context(ctx);
  crate::kernel::timer::interrupt();
  // Finished interrupt before switching
  INTERRUPT_CONTROLLER.finish(apic::INT_TIMER);
  core.clear_context();
}

extern "x86-interrupt" fn error_interrupt_handler(stack_frame: InterruptStackFrame) {
  error!("APIC LVT Error Interrupt");
  error!("ESR: {:#?}", unsafe { apic::local_apic().error_flags() });
  error!("{:#?}", stack_frame);
  INTERRUPT_CONTROLLER.finish(apic::ERROR_INTERRUPT_NUMBER as usize);
  loop {}
}

extern "x86-interrupt" fn spurious_interrupt_handler(stack_frame: InterruptStackFrame) {
  error!("Spurious Interrupt: {:#?}", stack_frame);
  loop {}
}

#[no_mangle]
extern "C" fn syscall_rust_entry(ctx: *mut ContextFrame) {
  let core = crate::kernel::cpu::cpu();
  core.set_context(ctx);
  crate::kernel::syscall::syscall();
  core.clear_context();
}
