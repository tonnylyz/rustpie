#![no_std]
#![no_main]

use core::fmt::Write;
use uefi::prelude::*;
use uefi::proto::loaded_image::LoadedImage;
use uefi::table::boot::{self, MemoryType};
use x86_64::structures::paging::{
  FrameAllocator, Mapper, OffsetPageTable, Page, PageTable, PageTableFlags, PhysFrame, Size2MiB,
  Size4KiB, Size1GiB,
};
use x86_64::{PhysAddr, VirtAddr};

#[macro_use]
mod macros {
  #[repr(C)] // guarantee 'bytes' comes after '_align'
  pub struct AlignedAs<Align, Bytes: ?Sized> {
    pub _align: [Align; 0],
    pub bytes: Bytes,
  }

  macro_rules! include_bytes_align_as {
    ($align_ty:ty, $path:literal) => {{
      // const block expression to encapsulate the static
      use $crate::macros::AlignedAs;

      // this assignment is made possible by CoerceUnsized
      static ALIGNED: &AlignedAs<$align_ty, [u8]> = &AlignedAs {
        _align: [],
        bytes: *include_bytes!($path),
      };

      &ALIGNED.bytes
    }};
  }
}

#[repr(align(4096))]
struct AlignPage;

const PA2KVA: u64 = 0xFFFF_8000_0000_0000;

fn page_table() -> OffsetPageTable<'static> {
  let p4_table_addr = x86_64::registers::control::Cr3::read()
    .0
    .start_address()
    .as_u64();
  let p4_table = unsafe { &mut *(p4_table_addr as *mut PageTable) };
  unsafe { OffsetPageTable::new(p4_table, VirtAddr::new(0)) }
}

fn identity_map(
  pa_max: u64,
  page_table: &mut impl Mapper<Size1GiB>,
  frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
  let start_frame = PhysFrame::containing_address(PhysAddr::new(0));
  let end_frame = PhysFrame::containing_address(PhysAddr::new(pa_max));
  for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
    let page = Page::containing_address(VirtAddr::new(frame.start_address().as_u64() + PA2KVA));
    let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    unsafe {
      page_table
        .map_to(page, frame, flags, frame_allocator)
        .unwrap()
        .flush();
    }
  }
}

#[uefi::entry]
fn efi_main(image: uefi::Handle, mut st: SystemTable<Boot>) -> Status {
  #[macro_export]
  macro_rules! eprint {
      ($($arg:tt)*) => (st.stdout().write_fmt(format_args!($($arg)*)));
  }

  #[macro_export]
  macro_rules! eprintln {
      () => ($crate::print!("\n"));
      ($($arg:tt)*) => ({
          st.stdout().write_fmt(format_args!("{}\n", format_args!($($arg)*))).unwrap();
      })
  }

  let mut pt = page_table();
  unsafe {
    x86_64::registers::control::Cr0::update(|f| {
      f.remove(x86_64::registers::control::Cr0Flags::WRITE_PROTECT)
    });
    x86_64::registers::model_specific::Efer::update(|f| {
      f.insert(x86_64::registers::model_specific::EferFlags::NO_EXECUTE_ENABLE)
    });
  }

  identity_map(
    0x1_0000_0000,
    &mut pt,
    &mut UEFIFrameAllocator(st.boot_services()),
  );

  let (image_base, image_size) = st
    .boot_services()
    .open_protocol_exclusive::<LoadedImage>(image)
    .unwrap()
    .info();
  eprintln!("image base {:X} size {:X}", image_base as usize, image_size);

  let boot_stack = st
    .boot_services()
    .allocate_pages(
      uefi::table::boot::AllocateType::AnyPages,
      MemoryType::BOOT_SERVICES_DATA,
      2,
    )
    .unwrap() as usize;
  let stack_top = boot_stack + 2 * 4096;
  eprintln!("boot stack top {:X}", stack_top);

  let boot_data = st
    .boot_services()
    .allocate_pages(
      uefi::table::boot::AllocateType::AnyPages,
      MemoryType::BOOT_SERVICES_DATA,
      1,
    )
    .unwrap() as usize as *mut rpabi::X64BootData;
  let boot_data = unsafe { boot_data.as_mut().unwrap() };

  let embedded_image = include_bytes_align_as!(AlignPage, "../../target/x86_64-virt-rustpi/release/rustpi.bin");
  let load_addr: usize = 0x1800000;
  let loaded_kernel = unsafe {
    core::slice::from_raw_parts_mut(load_addr as *mut u8, embedded_image.len())   
  };
  for i in 0..embedded_image.len() {
    loaded_kernel[i] = embedded_image[i];
  }
  eprintln!("kernel image base {:X} size {:X}", load_addr, embedded_image.len());

  let (_rt, mmap_iter) = st.exit_boot_services(MemoryType::BOOT_SERVICES_DATA);

  let mut largest_region: Option<&boot::MemoryDescriptor> = None;
  for desc in mmap_iter.entries() {
    if desc.ty == MemoryType::CONVENTIONAL {
      if let Some(d) = largest_region {
        if d.page_count < desc.page_count {
          largest_region = Some(desc)
        }
      } else {
        largest_region = Some(desc)
      }
    }
  }
  boot_data.free_mem_start = largest_region.unwrap().phys_start as usize;
  boot_data.free_mem_count = largest_region.unwrap().page_count as usize;
  let stack_top = PA2KVA as usize | stack_top;
  let entry = PA2KVA as usize | load_addr;
  let boot_data: usize = boot_data as *mut _ as usize;
  unsafe {
    core::arch::asm!("mov rsp, {}; call {}", in(reg) stack_top, in(reg) entry, in("rdi") boot_data);
  }
  Status::SUCCESS
}

struct UEFIFrameAllocator<'a>(&'a BootServices);

unsafe impl FrameAllocator<Size4KiB> for UEFIFrameAllocator<'_> {
  fn allocate_frame(&mut self) -> Option<PhysFrame> {
    let addr = self
      .0
      .allocate_pages(
        uefi::table::boot::AllocateType::AnyPages,
        MemoryType::LOADER_DATA,
        1,
      )
      .unwrap();
    let frame = PhysFrame::containing_address(PhysAddr::new(addr));
    Some(frame)
  }
}


#[panic_handler]
pub fn panic_handler(_: &core::panic::PanicInfo) -> ! { loop {} }
