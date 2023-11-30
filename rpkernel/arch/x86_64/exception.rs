use spin::Once;
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
use crate::MAX_CPU_NUMBER;

#[repr(C)]
#[derive(Debug, Clone)]
struct PerCpu {
  kernel_rsp: usize,
  user_rsp: usize,
  scratch: usize,
  gdt: GlobalDescriptorTable,
}

impl PerCpu {
  const fn new() -> Self {
    PerCpu {
      kernel_rsp: 0,
      user_rsp: 0,
      scratch: 0,
      gdt: GlobalDescriptorTable::new(),
    }
  }
}

static PER_CPU: Once<[PerCpu; MAX_CPU_NUMBER]> = Once::new();
static PER_CPU_TSS: Once<[TaskStateSegment; MAX_CPU_NUMBER]> = Once::new();
static PER_CPU_TSS_SEL: Once<[SegmentSelector; MAX_CPU_NUMBER]> = Once::new();

const KERNEL_CODE_SEGMENT_SEL: u16 = 0x8;
const KERNEL_STACK_SEGMENT_SEL: u16 = 0x10;
const USER_CODE_SEGMENT_SEL: u16 = 0x1B;
const USER_STACK_SEGMENT_SEL: u16 = 0x23;
const SYSRET_CODE_SEGMENT_SEL: u16 = 0x2B;

fn per_cpu_init() -> &'static [PerCpu; MAX_CPU_NUMBER] {
  const PER_CPU_DEFAULT: PerCpu = PerCpu::new();
  let mut per_cpu = [PER_CPU_DEFAULT; MAX_CPU_NUMBER];
  let mut per_cpu_tss = [TaskStateSegment::new(); MAX_CPU_NUMBER];
  let mut per_cpu_tss_sel = [SegmentSelector::NULL; MAX_CPU_NUMBER];

  for i in 0..MAX_CPU_NUMBER {
    // rustpi uses per-CPU kernel stack (not differentiate for threads)
    // the same stack is use for interrupt, syscall
    let kernel_stack = crate::kernel::stack::stack_of_core(i) as u64;
    per_cpu_tss[i].privilege_stack_table[0] = VirtAddr::new(kernel_stack); // interrupt kernel stack
    per_cpu_tss[i].interrupt_stack_table[0] = VirtAddr::new(kernel_stack); // interrupt kernel stack (when IST(0) is set in IDT)
  }

  let per_cpu_tss = PER_CPU_TSS.call_once(|| per_cpu_tss);
  for i in 0..MAX_CPU_NUMBER {
    let gdt = &mut per_cpu[i].gdt;
    let kernel_cs = gdt.add_entry(Descriptor::kernel_code_segment());
    let kernel_ss = gdt.add_entry(Descriptor::kernel_data_segment());
    let user_cs = gdt.add_entry(Descriptor::user_code_segment());
    let user_ss = gdt.add_entry(Descriptor::user_data_segment());
    let sysret_cs = gdt.add_entry(Descriptor::user_data_segment());
    assert_eq!(kernel_cs.0, KERNEL_CODE_SEGMENT_SEL);
    assert_eq!(kernel_ss.0, KERNEL_STACK_SEGMENT_SEL);
    assert_eq!(user_cs.0, USER_CODE_SEGMENT_SEL);
    assert_eq!(user_ss.0, USER_STACK_SEGMENT_SEL);
    assert_eq!(sysret_cs.0, SYSRET_CODE_SEGMENT_SEL);

    let kernel_stack = crate::kernel::stack::stack_of_core(i) as u64;
    per_cpu[i].kernel_rsp = kernel_stack as usize; // syscall kernel stack (retrieved after SWAPGS indexed by GS)

    let tss_selector = gdt.add_entry(Descriptor::tss_segment(&per_cpu_tss[i]));
    per_cpu_tss_sel[i] = tss_selector;
  }
  PER_CPU_TSS_SEL.call_once(|| per_cpu_tss_sel);
  PER_CPU.call_once(|| per_cpu)
}

static IDT: Once<InterruptDescriptorTable> = Once::new();

fn idt_init() -> InterruptDescriptorTable {
  extern "C" {
    // see interrupt.S
    fn timer_interrupt_handler();
  }
  let mut idt = InterruptDescriptorTable::new();
  set_general_handler!(&mut idt, abort, 0..32);
  set_general_handler!(&mut idt, unhandle, 32..64);
  set_general_handler!(&mut idt, unknown, 64..);
  // we don't allow nested interrupt in rustpi, thus using same stack index is safe
  unsafe {
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
    idt[apic::INT_TIMER]
      .set_handler_addr(VirtAddr::new(timer_interrupt_handler as u64))
      .set_stack_index(0);
    idt[apic::ERROR_INTERRUPT_NUMBER as usize]
      .set_handler_fn(error_interrupt_handler)
      .set_stack_index(0);
    idt[apic::SPURIOUS_INTERRUPT_NUMBER as usize]
      .set_handler_fn(spurious_interrupt_handler)
      .set_stack_index(0);
  }
  idt
}

pub fn init() {
  extern "C" {
    // see interrupt.S
    fn syscall_entry();
  }
  unsafe {
    // Set MSR EFER
    //  - enable syscall extensions
    //  - disable no execute TODO: enable it
    Efer::update(|f: &mut EferFlags| {
      f.insert(EferFlags::SYSTEM_CALL_EXTENSIONS);
      f.remove(EferFlags::NO_EXECUTE_ENABLE);
    });

    // Set CR4
    //  - enable PCID
    Cr4::update(|f| f.insert(Cr4Flags::PCID));

    // Setup GDT; Load GDT (every CPU)
    per_cpu_init()[crate::core_id()].gdt.load();
    // init all kernel segment selector. Must be done after LGDT
    CS::set_reg(SegmentSelector(0x8));
    SS::set_reg(SegmentSelector(0x10));
    DS::set_reg(SegmentSelector::NULL);
    ES::set_reg(SegmentSelector::NULL);
    FS::set_reg(SegmentSelector::NULL);
    GS::set_reg(SegmentSelector::NULL);

    // make SWAPGS work here. Supposed we are in kernel, set (GS, KernelGS) to (per_cpu_va, 0). Before entering ring-3, it's swapped back.
    let per_cpu = &PER_CPU.get().unwrap()[crate::core_id()];
    GsBase::write(VirtAddr::new(per_cpu as *const _ as u64));
    KernelGsBase::write(VirtAddr::new(0));

    // Load TSS, get tss selector from per-cpu data
    x86_64::instructions::tables::load_tss(PER_CPU_TSS_SEL.get().unwrap()[crate::core_id()]);

    // Setup IDT (bsp CPU only); Load IDT (every CPU)
    IDT.call_once(|| idt_init()).load();

    // Syscall
    //  - set kernel CS/SS when entering kernel by syscall
    //  - currently rustpi doesn't use sysret to return to user-space
    Star::write(
      SegmentSelector(SYSRET_CODE_SEGMENT_SEL),
      SegmentSelector(USER_STACK_SEGMENT_SEL),
      SegmentSelector(KERNEL_CODE_SEGMENT_SEL),
      SegmentSelector(KERNEL_STACK_SEGMENT_SEL),
    )
    .unwrap();
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
  println!(
    "{:?} index {} error_code {:?}",
    stack_frame, index, error_code
  );
  loop {}
}
fn unhandle(stack_frame: InterruptStackFrame, index: u8, error_code: Option<u64>) {
  println!("unhandle");
  println!(
    "{:?} index {} error_code {:?}",
    stack_frame, index, error_code
  );
  loop {}
}
fn unknown(stack_frame: InterruptStackFrame, index: u8, error_code: Option<u64>) {
  println!("unknown");
  println!(
    "{:?} index {} error_code {:?}",
    stack_frame, index, error_code
  );
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
