use rpabi::{
  platform::PlatformInfo,
  X64BootData,
};
use spin::Once;
use x86_64::{
  instructions::port::Port,
  structures::gdt::Descriptor,
  VirtAddr,
};

use crate::{
  arch::PAGE_SIZE,
  kernel::{stack::stack_of_core, traits::Address},
};

static CPU_NUMBER: Once<usize> = Once::new();
pub fn cpu_number() -> usize {
  *CPU_NUMBER.get().unwrap()
}

pub fn init_per_core() {}

pub fn core_id() -> usize {
  0
}

pub fn launch_other_cores() {}

pub static PLATFORM_INFO: Once<PlatformInfo> = Once::new();

pub struct I8250 {}

pub static DEBUG_UART: Once<I8250> = Once::initialized(I8250 {});

impl crate::kernel::print::DebugUart for I8250 {
  fn init(&self) {}

  fn putc(&self, c: u8) {
    unsafe {
      const COM1: u16 = 0x3f8;
      // todo: poll LSR (COM1+5)
      Port::new(COM1 + 0).write(c);
    }
  }

  fn getc(&self) -> Option<u8> {
    None
  }
}

static mut GDT: x86_64::structures::gdt::GlobalDescriptorTable =
  x86_64::structures::gdt::GlobalDescriptorTable::new();
static mut TSS: x86_64::structures::tss::TaskStateSegment =
  x86_64::structures::tss::TaskStateSegment::new();

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

pub fn init(boot_data: usize) -> (core::ops::Range<usize>, core::ops::Range<usize>) {
  let boot_data = unsafe {
    ((boot_data.pa2kva()) as *const X64BootData)
      .as_ref()
      .unwrap()
  };
  let free_mib = boot_data.free_mem_count * PAGE_SIZE / 1024 / 1024;
  println!("Free memory {} MiB", free_mib);

  extern "C" {
    fn KERNEL_END();
  }

  let kernel_end = crate::util::round_up((KERNEL_END as usize).kva2pa(), PAGE_SIZE);
  let heap_start = kernel_end;
  let heap_end = heap_start + crate::mm::config::CONFIG_NON_PAGED_MEMORY_SIZE;
  let paged_start = heap_end;
  let paged_end = boot_data.free_mem_start + boot_data.free_mem_count * PAGE_SIZE;
  unsafe {
    use x86_64::registers::segmentation::*;

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

    x86_64::registers::model_specific::Efer::update(|f: &mut x86_64::registers::model_specific::EferFlags| {
      f.insert(x86_64::registers::model_specific::EferFlags::SYSTEM_CALL_EXTENSIONS);
      f.remove(x86_64::registers::model_specific::EferFlags::NO_EXECUTE_ENABLE)
    });
    let efer = x86_64::registers::model_specific::Efer::read();
    println!("efer {:?}", efer);
    x86_64::registers::control::Cr4::update(|f| {
      f.insert(x86_64::registers::control::Cr4Flags::PCID)
    });
    let cr4 = x86_64::registers::control::Cr4::read();
    println!("cr4 {:?}", cr4);
    TSS.privilege_stack_table[0] = VirtAddr::new(stack_of_core(0) as u64);
    X64_PER_CPU_DATA.kernel_rsp = stack_of_core(0);
    GDT.load();
    CS::set_reg(kernel_cs);
    SS::set_reg(kernel_ss);
    DS::set_reg(SegmentSelector(0));
    ES::set_reg(SegmentSelector(0));
    FS::set_reg(SegmentSelector(0));
    GS::set_reg(SegmentSelector(0));
    let per_cpu = VirtAddr::new(&X64_PER_CPU_DATA as *const _ as u64);
    println!("per cpu {:?}", per_cpu);
    x86_64::registers::model_specific::GsBase::write(per_cpu);
    x86_64::registers::model_specific::KernelGsBase::write(VirtAddr::new(0));
    x86_64::registers::model_specific::Star::write(
      sysret_cs,
      user_ss,
      kernel_cs,
      kernel_ss,
    ).unwrap();
    x86_64::instructions::tables::load_tss(tss_selector);
  }

  CPU_NUMBER.call_once(|| 1);
  (heap_start..heap_end, paged_start..paged_end)
}
