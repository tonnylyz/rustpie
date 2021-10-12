#![no_std]
#![feature(global_asm)]
#![feature(trait_alias)]
#![feature(core_intrinsics)]

extern crate alloc;
#[macro_use]
extern crate log;

use alloc::boxed::Box;

use fallible_iterator::FallibleIterator;
use gimli::{
  BaseAddresses,
  CfaRule,
  EndianSlice,
  FrameDescriptionEntry,
  Pointer,
  UninitializedUnwindContext,
  UnwindSection,
  UnwindTableRow,
  read::RegisterRule
};

use registers::{
  LandingRegisters,
  SavedRegs,
  Aarch64,
  Registers
};

pub mod registers;
pub mod elf;
mod lsda;
pub mod catch;

pub struct UnwindingContext {
  stack_frame_iter: StackFrameIter,
}

#[derive(Debug)]
pub struct StackFrame {
  personality: Option<u64>,
  lsda: Option<u64>,
  initial_address: u64,
  call_site_address: u64,
}

#[derive(Debug)]
pub struct StackFrameIter {
  registers: Registers,
  state: Option<(UnwindRowReference, u64)>,
}

impl StackFrameIter {
  pub fn new(registers: Registers) -> Self {
    StackFrameIter {
      registers,
      state: None,
    }
  }
}

impl FallibleIterator for StackFrameIter {
  type Item = StackFrame;
  type Error = &'static str;

  fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
    let registers = &mut self.registers;
    if let Some((unwind_row_ref, cfa)) = self.state.take() {
      let mut new_regs = registers.clone();
      new_regs[Aarch64::X30] = None;
      new_regs[Aarch64::SP] = Some(cfa);
      unwind_row_ref.with_unwind_info(|_fde, row| {
        for &(reg_num, ref rule) in row.registers() {
          if reg_num == Aarch64::SP {
            continue;
          }
          if reg_num == Aarch64::X30 {
            continue;
          }
          new_regs[reg_num] = match *rule {
            RegisterRule::Undefined
            | RegisterRule::Expression(_)
            | RegisterRule::ValExpression(_)
            | RegisterRule::Architectural => {
              error!("unsupported rule {:?}", *rule);
              return Err("unsupported rule")
            },
            RegisterRule::SameValue => registers[reg_num],
            RegisterRule::Register(other) => registers[other],
            RegisterRule::Offset(offset) => {
              let value = unsafe {
                (cfa.wrapping_add(offset as u64) as *const u64).read()
              };
              Some(value)
            }
            RegisterRule::ValOffset(offset) => Some(cfa.wrapping_add(offset as u64)),
          };
        }
        Ok(())
      })?;
      *registers = new_regs;
    }

    let return_address = match registers[Aarch64::X30] {
      Some(0) | None => return Ok(None),
      Some(ra) => ra,
    };
    let caller = return_address - 4;
    let (eh_frame_sec, base_addrs) = get_eh_frame_info();

    let row_ref = UnwindRowReference { caller, eh_frame_sec, base_addrs };
    let (cfa, frame) = row_ref.with_unwind_info(|fde, row| {
      let cfa = match *row.cfa() {
        CfaRule::RegisterAndOffset { register, offset } => {
          let reg_value = registers[register].ok_or_else(|| {
            error!("cfa reg none");
            "cfa reg none"
          })?;
          reg_value.wrapping_add(offset as u64)
        }
        CfaRule::Expression(_) => {
          error!("cfa expression");
          return Err("cfa expression");
        }
      };

      let frame = StackFrame {
        personality: None,
        lsda: fde.lsda().map(|x| unsafe {
          match x {
            Pointer::Direct(x) => x,
            Pointer::Indirect(x) => *(x as *const u64),
          }
        }),
        initial_address: fde.initial_address(),
        call_site_address: caller,
      };
      Ok((cfa, frame))
    })?;

    self.state = Some((row_ref, cfa));
    Ok(Some(frame))
  }
}

pub trait FuncWithRegisters = Fn(Registers) -> Result<(), &'static str>;

type RefFuncWithRegisters<'a> = &'a dyn FuncWithRegisters;

pub fn invoke_with_current_registers<F>(f: F) -> Result<(), &'static str>
  where F: FuncWithRegisters
{
  let f: RefFuncWithRegisters = &f;
  let result = unsafe {
    let res_ptr = unwind_trampoline(&f);
    let res_boxed = Box::from_raw(res_ptr);
    *res_boxed
  };
  return result;
}

#[allow(improper_ctypes)] // trampoline just forwards the ptr
extern "C" {
  fn unwind_trampoline(_func: *const RefFuncWithRegisters) -> *mut Result<(), &'static str>;
  fn unwind_lander(regs: *const LandingRegisters) -> !;
}

global_asm! {
r#"
.global unwind_trampoline
unwind_trampoline:
.cfi_startproc
     mov x1, sp
     sub sp, sp, 0xA0
     .cfi_adjust_cfa_offset 0xA0
     stp x19, x20, [sp, #0x00]
     stp x21, x22, [sp, #0x10]
     stp x23, x24, [sp, #0x20]
     stp x25, x26, [sp, #0x30]
     stp x27, x28, [sp, #0x40]
     stp x29, lr,  [sp, #0x50]
     .cfi_rel_offset lr, 0x58
     //stp d8,  d9,  [sp, #0x60]
     //stp d10, d11, [sp, #0x70]
     //stp d12, d13, [sp, #0x80]
     //stp d14, d15, [sp, #0x90]
     mov x2, sp
     bl unwind_recorder
     ldr lr, [sp, #0x58]
     .cfi_restore lr
     add sp, sp, 0xA0
     .cfi_adjust_cfa_offset -0xA0
     ret
.cfi_endproc
.global unwind_lander
unwind_lander:
     ldp x2,  x3,  [x0, #0x010]
     ldp x4,  x5,  [x0, #0x020]
     ldp x6,  x7,  [x0, #0x030]
     ldp x8,  x9,  [x0, #0x040]
     ldp x10, x11, [x0, #0x050]
     ldp x12, x13, [x0, #0x060]
     ldp x14, x15, [x0, #0x070]
     ldp x16, x17, [x0, #0x080]
     ldp x18, x19, [x0, #0x090]
     ldp x20, x21, [x0, #0x0A0]
     ldp x22, x23, [x0, #0x0B0]
     ldp x24, x25, [x0, #0x0C0]
     ldp x26, x27, [x0, #0x0D0]
     ldp x28, x29, [x0, #0x0E0]
     ldp x30, x1,  [x0, #0x0F0]
     mov sp, x1
     //ldp d0,  d1,  [x0, #0x100]
     //ldp d2,  d3,  [x0, #0x110]
     //ldp d4,  d5,  [x0, #0x120]
     //ldp d6,  d7,  [x0, #0x130]
     //ldp d8,  d9,  [x0, #0x140]
     //ldp d10, d11, [x0, #0x150]
     //ldp d12, d13, [x0, #0x160]
     //ldp d14, d15, [x0, #0x170]
     //ldp d16, d17, [x0, #0x180]
     //ldp d18, d19, [x0, #0x190]
     //ldp d20, d21, [x0, #0x1A0]
     //ldp d22, d23, [x0, #0x1B0]
     //ldp d24, d25, [x0, #0x1C0]
     //ldp d26, d27, [x0, #0x1D0]
     //ldp d28, d29, [x0, #0x1E0]
     //ldp d30, d31, [x0, #0x1F0]
     ldp x0,  x1,  [x0, #0x000]
     ret x30 // HYPERSPACE JUMP :D
"#
}

#[no_mangle]
pub unsafe extern "C" fn unwind_recorder(
  func: *const RefFuncWithRegisters,
  stack: u64,
  saved_regs: *mut SavedRegs,
) -> *mut Result<(), &'static str> {
  let func = &*func;
  let saved_regs = &*saved_regs;

  let mut registers = Registers::default();

  registers[Aarch64::X19] = Some(saved_regs.r[0]);
  registers[Aarch64::X20] = Some(saved_regs.r[1]);
  registers[Aarch64::X21] = Some(saved_regs.r[2]);
  registers[Aarch64::X22] = Some(saved_regs.r[3]);
  registers[Aarch64::X23] = Some(saved_regs.r[4]);
  registers[Aarch64::X24] = Some(saved_regs.r[5]);
  registers[Aarch64::X25] = Some(saved_regs.r[6]);
  registers[Aarch64::X26] = Some(saved_regs.r[7]);
  registers[Aarch64::X27] = Some(saved_regs.r[8]);
  registers[Aarch64::X28] = Some(saved_regs.r[9]);
  registers[Aarch64::X29] = Some(saved_regs.r[10]);
  registers[Aarch64::SP] = Some(stack);
  registers[Aarch64::X30] = Some(saved_regs.lr);

  let res = func(registers);
  Box::into_raw(Box::new(res))
}

pub unsafe fn land(regs: &Registers, landing_pad_address: u64) {
  let mut lr = LandingRegisters {
    x: [0; 29],
    fp: regs[Aarch64::X29].unwrap_or(0),
    lr: landing_pad_address,
    sp: regs[Aarch64::SP].unwrap_or(0),
  };

  lr.x[0] = regs[Aarch64::X0].unwrap_or(0);
  lr.x[1] = regs[Aarch64::X1].unwrap_or(0);
  lr.x[2] = regs[Aarch64::X2].unwrap_or(0);
  lr.x[3] = regs[Aarch64::X3].unwrap_or(0);
  lr.x[4] = regs[Aarch64::X4].unwrap_or(0);
  lr.x[5] = regs[Aarch64::X5].unwrap_or(0);
  lr.x[6] = regs[Aarch64::X6].unwrap_or(0);
  lr.x[7] = regs[Aarch64::X7].unwrap_or(0);
  lr.x[8] = regs[Aarch64::X8].unwrap_or(0);
  lr.x[9] = regs[Aarch64::X9].unwrap_or(0);
  lr.x[10] = regs[Aarch64::X10].unwrap_or(0);
  lr.x[11] = regs[Aarch64::X11].unwrap_or(0);
  lr.x[12] = regs[Aarch64::X12].unwrap_or(0);
  lr.x[13] = regs[Aarch64::X13].unwrap_or(0);
  lr.x[14] = regs[Aarch64::X14].unwrap_or(0);
  lr.x[15] = regs[Aarch64::X15].unwrap_or(0);
  lr.x[16] = regs[Aarch64::X16].unwrap_or(0);
  lr.x[17] = regs[Aarch64::X17].unwrap_or(0);
  lr.x[18] = regs[Aarch64::X18].unwrap_or(0);
  lr.x[19] = regs[Aarch64::X19].unwrap_or(0);
  lr.x[20] = regs[Aarch64::X20].unwrap_or(0);
  lr.x[21] = regs[Aarch64::X21].unwrap_or(0);
  lr.x[22] = regs[Aarch64::X22].unwrap_or(0);
  lr.x[23] = regs[Aarch64::X23].unwrap_or(0);
  lr.x[24] = regs[Aarch64::X24].unwrap_or(0);
  lr.x[25] = regs[Aarch64::X25].unwrap_or(0);
  lr.x[26] = regs[Aarch64::X26].unwrap_or(0);
  lr.x[27] = regs[Aarch64::X27].unwrap_or(0);
  lr.x[28] = regs[Aarch64::X28].unwrap_or(0);

  unwind_lander(&lr);
}

type NativeEndianSliceReader<'i> = EndianSlice<'i, gimli::NativeEndian>;

#[derive(Debug)]
struct UnwindRowReference {
  caller: u64,
  eh_frame_sec: &'static [u8],
  base_addrs: BaseAddresses,
}

impl UnwindRowReference {
  fn with_unwind_info<O, F>(&self, mut f: F) -> Result<O, &'static str>
    where F: FnMut(&FrameDescriptionEntry<NativeEndianSliceReader, usize>, &UnwindTableRow<NativeEndianSliceReader>) -> Result<O, &'static str>
  {
    let eh_frame = gimli::read::EhFrame::new(
      self.eh_frame_sec,
      gimli::NativeEndian,
    );
    // println!("{:#X?}", self);
    let mut unwind_ctx = UninitializedUnwindContext::new();
    let fde = eh_frame.fde_for_address(&self.base_addrs, self.caller, gimli::EhFrame::cie_from_offset).map_err(|_e| {
      error!("gimli error: {:?}", _e);
      "gimli error while finding FDE for address"
    })?;

    let unwind_table_row = fde.unwind_info_for_address(&eh_frame, &self.base_addrs, &mut unwind_ctx, self.caller).map_err(|_e| {
      error!("gimli error: {:?}", _e);
      "gimli error while finding unwind info for address"
    })?;

    f(&fde, &unwind_table_row)
  }
}

fn get_eh_frame_info() -> (&'static [u8], BaseAddresses) {
  let ehf = elf::section_by_name(".eh_frame").unwrap();
  let eh_frame = ehf.start;
  let eh_frame_hdr = elf::section_by_name(".eh_frame_hdr").unwrap().start;
  let text = elf::section_by_name(".text").unwrap().start;
  // let got = elf::section_by_name(".got").unwrap().start;
  let base_addrs = BaseAddresses::default()
    .set_eh_frame_hdr(eh_frame_hdr as u64)
    .set_eh_frame(eh_frame as u64)
    .set_text(text as u64);
    // .set_got(got as u64);

  (unsafe { core::slice::from_raw_parts(
    ehf.start as usize as *const u8,
    (ehf.end - ehf.start) as usize
  ) }, base_addrs)
}

pub fn start_unwinding_from_exception(registers: Registers) {
  let ctx = Box::into_raw(Box::new(UnwindingContext {
    stack_frame_iter: StackFrameIter::new(registers)
  }));
  continue_unwinding(ctx);
  cleanup(ctx);
}

pub fn start_unwinding(stack_frames_to_skip: usize) {
  let ctx = Box::into_raw(Box::new(UnwindingContext {
    stack_frame_iter: StackFrameIter::new(Registers::default())
  }));

  let _ = invoke_with_current_registers(|registers| {
    let unwinding_context = unsafe {
      &mut *ctx
    };
    unwinding_context.stack_frame_iter.registers = registers;
    for _i in 0..stack_frames_to_skip {
      let _ = unwinding_context.stack_frame_iter.next();
    }
    continue_unwinding(ctx)
  });
  cleanup(ctx);
}

fn continue_unwinding(unwinding_context_ptr: *mut UnwindingContext) -> Result<(), &'static str> {
  let stack_frame_iter = unsafe { &mut (*unwinding_context_ptr).stack_frame_iter };

  let (mut regs, landing_pad_address) =
    match stack_frame_iter.next()? {
      None => {
        error!("no frame left");
        return Ok(());
      }
      Some(frame) => {
        info!("function addr {:016x}", frame.initial_address);
        info!("call site {:016x}", frame.call_site_address);
        match frame.lsda {
          None => {
            error!("frame has no lsda");
            return continue_unwinding(unwinding_context_ptr);
          }
          Some(lsda) => {
            match elf::section_by_addr(lsda as usize) {
              None => {
                error!("cannot find lsda in elf");
                return Err("cannot find lsda in elf");
              }
              Some(lsda_slice) => {
                let table = lsda::GccExceptTableArea::new(lsda_slice, gimli::NativeEndian, frame.initial_address);
                let entry = match table.call_site_table_entry_for_address(frame.call_site_address) {
                  Ok(x) => x,
                  Err(e) => {
                    error!("call site has no entry {:016x} {}", frame.call_site_address, e);

                    // Now we don't have an exact match. We try to use the previous
                    let mut iter = table.call_site_table_entries().map_err(|_e| { "Couldn't find call_site_table_entries" })?;

                    let mut closest_entry = None;
                    while let Some(entry) = iter.next().map_err(|_e| { "Couldn't iterate through the entries" })? {
                      if entry.range_of_covered_addresses().start < frame.call_site_address {
                        closest_entry = Some(entry);
                      }
                    }

                    if let Some(closest_entry) = closest_entry {
                      closest_entry
                    } else {
                      error!("no closest entry");
                      return Err("no closest entry");
                    }
                  }
                };
                (stack_frame_iter.registers.clone(), entry.landing_pad_address())
              }
            }
          }
        }
      }
    };

  let landing_pad_address = match landing_pad_address {
    Some(lpa) => lpa,
    _ => {
      warn!("frame has lsda but no landing pad");
      return continue_unwinding(unwinding_context_ptr);
    }
  };

  unsafe {
    // brk #?
    if (landing_pad_address as usize as *const u32).read() & 0xFF_E0_00_00 == 0xd4_20_00_00 {
      error!("land at {:016x} is `brk #?`", landing_pad_address);
      return continue_unwinding(unwinding_context_ptr);
    }
  }

  info!("land at {:016x}", landing_pad_address);
  regs[Aarch64::X0] = Some(unwinding_context_ptr as u64);
  unsafe {
    land(&regs, landing_pad_address);
  }
  Err("should not return")
}

fn cleanup(unwinding_context_ptr: *mut UnwindingContext) {
  let unwinding_context_boxed = unsafe { Box::from_raw(unwinding_context_ptr) };
  let unwinding_context = *unwinding_context_boxed;
  drop(unwinding_context.stack_frame_iter);
}

pub fn unwind_resume(unwinding_context_ptr: usize) -> ! {
  let unwinding_context_ptr = unwinding_context_ptr as *mut UnwindingContext;

  continue_unwinding(unwinding_context_ptr);
  cleanup(unwinding_context_ptr);
  error!("unwind_resume end");
  loop {}
}
