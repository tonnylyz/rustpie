use alloc::rc::Rc;
use core::panic::PanicInfo;

use addr2line;
use addr2line::Context;
use addr2line::gimli;
use backtracer;
use spin::Once;

use crate::arch::ContextFrameTrait;
use crate::lib::core::CoreTrait;

#[inline(always)]
pub fn backtrace_exception_no_resolve() {
  println!("Backtrace:");

  let ctx = crate::lib::core::current().context();

  backtracer::trace_from(
    backtracer::EntryPoint::new(
      ctx.gpr(29) as u64,
      ctx.stack_pointer() as u64,
      ctx.exception_pc() as u64,
      ctx.gpr(30) as u64),
    |frame| {
      let ip = frame.ip();
      println!("ip:{:?}", ip);
      true
    },
  );
}

#[inline(always)]
pub fn backtrace_exception() {
  println!("Backtrace:");
  unsafe {
    if ELF_CONTEXT.is_none() {
      println!("ELF_CONTEXT was not initialized");
      backtrace_exception_no_resolve();
      return;
    }
  }
  let relocated_offset = RELOCATED_OFFSET;
  let mut count = 0;

  let ctx = crate::lib::core::current().context();

  unsafe {
    backtracer::trace_from(backtracer::EntryPoint::new(
      ctx.gpr(29) as u64,
      ctx.stack_pointer() as u64,
      ctx.exception_pc() as u64,
      ctx.gpr(30) as u64),
                           |frame| {
                             count += 1;
                             backtrace_format(ELF_CONTEXT.as_ref(), relocated_offset, count, frame)
                           },
    );
  }
}

pub fn backtrace_no_resolve() {
  backtracer::trace(|frame| {
    let ip = frame.ip();
    println!("ip:{:?}", ip);
    true
  });
}

static ELF_DATA: Once<&'static [u8]> = Once::new();
// #[thread_local]
// static ELF_CONTEXT: Once<Option<Context>> = Once::new();
static mut ELF_CONTEXT: Option<Context> = None;
static ELF_BIN: Once<elfloader::ElfBinary> = Once::new();
static RELOCATED_OFFSET: u64 = 0x0;

pub fn init_backtrace(elf_data: &'static [u8]) {
  ELF_DATA.call_once(|| elf_data);
}

pub fn init_backtrace_context() {
  ELF_BIN.call_once(|| {
    let elf_data = ELF_DATA.get().expect("ELF_DATA was not initialized");
    let elf_binary =
      elfloader::ElfBinary::new("kernel", &elf_data).expect("Can't parse kernel binary.");
    elf_binary
  });

  let elf_binary = ELF_BIN.get().expect("ELF_BIN was not initialized");
  if let Some(context) = new_ctxt(&elf_binary) {
    unsafe {
      ELF_CONTEXT = Some(context);
    }
  }
}

fn new_ctxt(file: &elfloader::ElfBinary) -> Option<Context> {
  let endian = gimli::RunTimeEndian::Little;

  fn load_section<S, Endian>(elf: &elfloader::ElfBinary, endian: Endian) -> S
    where
      S: gimli::Section<gimli::EndianRcSlice<Endian>>,
      Endian: gimli::Endianity,
  {
    let data = elf
      .file
      .find_section_by_name(S::section_name())
      .map(|s| s.raw_data(&elf.file))
      .unwrap_or(&[]);
    S::from(gimli::EndianRcSlice::new(Rc::from(&*data), endian))
  }

  let debug_abbrev: gimli::DebugAbbrev<_> = load_section(file, endian);
  let debug_addr: gimli::DebugAddr<_> = load_section(file, endian);
  let debug_info: gimli::DebugInfo<_> = load_section(file, endian);
  let debug_line: gimli::DebugLine<_> = load_section(file, endian);
  let debug_line_str: gimli::DebugLineStr<_> = load_section(file, endian);
  let debug_ranges: gimli::DebugRanges<_> = load_section(file, endian);
  let debug_rnglists: gimli::DebugRngLists<_> = load_section(file, endian);
  let debug_str: gimli::DebugStr<_> = load_section(file, endian);
  let debug_str_offsets: gimli::DebugStrOffsets<_> = load_section(file, endian);
  let default_section = gimli::EndianRcSlice::new(Rc::from(&[][..]), endian);

  Context::from_sections(
    debug_abbrev,
    debug_addr,
    debug_info,
    debug_line,
    debug_line_str,
    debug_ranges,
    debug_rnglists,
    debug_str,
    debug_str_offsets,
    default_section,
  )
    .ok()
}

fn backtrace_format(
  context: Option<&Context>,
  relocated_offset: u64,
  count: usize,
  frame: &backtracer::Frame,
) -> bool {
  let ip = frame.ip();
  println!("frame #{:<2} - {:#02$x}", count, ip as usize, 20);
  let mut resolved = false;

  backtracer::resolve(context, relocated_offset, ip, |symbol| {
    if !resolved {
      resolved = true;
    } else {
      print!("                                ");
    }
    if let Some(name) = symbol.name() {
      if name.as_bytes().len() == 0 {
        print!(" - <empty>");
      } else {
        print!(" - {}", name);
        if let Some(file) = symbol.filename() {
          print!(" ({}", file);
          if let Some(line) = symbol.lineno() {
            print!(":{})", line);
          } else {
            print!(")");
          }
        }
      }
    } else {
      //println!(" - <unknown>");
    }
    println!();
  });

  if !resolved {
    println!(" - <no info>");
  }
  true
}

#[inline(always)]
pub fn backtrace() {
  unsafe {
    if ELF_CONTEXT.is_none() {
      println!("ELF_CONTEXT was not initialized");
      backtrace_no_resolve();
      return;
    }
  }
  println!("Backtrace:");

  let relocated_offset = RELOCATED_OFFSET;

  let mut count = 0;
  unsafe {
    backtracer::trace(|frame| {
      count += 1;
      backtrace_format(ELF_CONTEXT.as_ref(), relocated_offset, count, frame)
    });
  }
}

#[cfg_attr(target_os = "none", panic_handler)]
#[no_mangle]
pub fn panic_impl(info: &PanicInfo) -> ! {
  println!("[Panic]");
  if let Some(message) = info.message() {
    println!("\tMessage: {}", message);
  }
  if let Some(location) = info.location() {
    println!("\tLocation: {}:{}", location.file(), location.line());
  } else {
    println!();
  }

  // backtrace();

  loop {}
}
