mod registers;
pub mod elf;
mod lsda;
pub mod catch;

use registers::Registers;
use fallible_iterator::FallibleIterator;
use gimli::{BaseAddresses, FrameDescriptionEntry, UnwindTableRow, UninitializedUnwindContext, UnwindSection, CfaRule, Pointer, EndianSlice};
use gimli::SectionId::EhFrame;
use gimli::read::RegisterRule;
use alloc::boxed::Box;
use crate::unwind::registers::{SavedRegs, LandingRegisters};
use registers::Aarch64;

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
  // namespace: CrateNameSpace,
  registers: Registers,
  state: Option<(UnwindRowReference, u64)>,
  cfa_adjustment: Option<i64>,
  last_frame_was_exception_handler: bool,
}

impl StackFrameIter {
  pub fn new(registers: Registers) -> Self {
    StackFrameIter {
      registers,
      state: None,
      cfa_adjustment: None,
      last_frame_was_exception_handler: false,
    }
  }
}

impl FallibleIterator for StackFrameIter {
  type Item = StackFrame;
  type Error = &'static str;

  fn next(&mut self) -> Result<Option<Self::Item>, Self::Error> {
    let registers = &mut self.registers;
    let prev_cfa_adjustment = self.cfa_adjustment;
    if let Some((unwind_row_ref, cfa)) = self.state.take() {
      let mut new_regs = registers.clone();
      new_regs[Aarch64::X30] = None;
      new_regs[Aarch64::SP] = Some(cfa);
      if let Some(extra_offset) = prev_cfa_adjustment {
        new_regs[Aarch64::SP] = Some(cfa.wrapping_add(extra_offset as u64));
      }
      unwind_row_ref.with_unwind_info(|_fde, row| {
        for &(reg_num, ref rule) in row.registers() {
          if reg_num == Aarch64::SP {
            continue;
          }
          if reg_num == Aarch64::X30 {
            if let Some(pca) = prev_cfa_adjustment {
              info!("pca {:x}", pca);
              continue;
            }
          }
          new_regs[reg_num] = match *rule {
            RegisterRule::Undefined
            | RegisterRule::Expression(_)
            | RegisterRule::ValExpression(_)
            | RegisterRule::Architectural => {
              error!("{:?}", *rule);
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
      // info!("old {:X?}", registers);
      // info!("new {:X?}", new_regs);
      *registers = new_regs;
    }

    let return_address = match registers[Aarch64::X30] {
      Some(0) | None => return Ok(None),
      Some(ra) => ra,
    };
    let caller = return_address - 4;

    let (eh_frame_sec, base_addrs) = get_eh_frame_info();
    // info!("{:#X?}", base_addrs);
    let mut cfa_adjustment = None;
    let mut this_frame_is_exception_handler = false;
    let row_ref = UnwindRowReference { caller, eh_frame_sec, base_addrs };
    let (cfa, frame) = row_ref.with_unwind_info(|fde, row| {
      let cfa = match *row.cfa() {
        CfaRule::RegisterAndOffset { register, offset } => {
          let reg_value = registers[register].ok_or_else(|| {
            error!("CFA rule specified register {:?} with offset {:#X}, but register {:?}({}) had no value!", register, offset, register, register.0);
            "CFA rule specified register with offset, but that register had no value."
          })?;
          reg_value.wrapping_add(offset as u64)
        }
        CfaRule::Expression(_expr) => {
          error!("CFA rules based on Expressions are not yet supported. Expression: {:?}", _expr);
          return Err("CFA rules based on Expressions are not yet supported.");
        }
      };

      cfa_adjustment = None;

      let frame = StackFrame {
        personality: None,
        lsda: fde.lsda().map(|x| unsafe { deref_ptr(x) }),
        initial_address: fde.initial_address(),
        call_site_address: caller,
      };
      Ok((cfa, frame))
    })?;
    self.cfa_adjustment = cfa_adjustment;
    self.last_frame_was_exception_handler = this_frame_is_exception_handler;
    self.state = Some((row_ref, cfa));
    Ok(Some(frame))
  }
}

unsafe fn deref_ptr(ptr: Pointer) -> u64 {
  match ptr {
    Pointer::Direct(x) => x,
    Pointer::Indirect(x) => *(x as *const u64),
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

  // for (regnum, v) in saved_regs.vector_half.iter().enumerate() {
  //   registers[DwarfRegisterAArch64::V8 as u8 + regnum as u8] = Some(*v);
  // }
  registers[Aarch64::SP] = Some(stack);
  registers[Aarch64::X30] = Some(saved_regs.lr);

  let res = func(registers);
  Box::into_raw(Box::new(res))
}

pub unsafe fn land(regs: &Registers, landing_pad_address: u64) -> Result<(), &'static str> {
  let mut lr = LandingRegisters {
    r: [0; 29],
    fp: regs[Aarch64::X29].unwrap_or(0),
    // lr: regs[Aarch64::X30].unwrap_or(0),
    lr: landing_pad_address,
    sp: regs[Aarch64::SP].unwrap_or(0),
    //vector_half: [0; 32]
  };

  lr.r[0] = regs[Aarch64::X0].unwrap_or(0);
  lr.r[1] = regs[Aarch64::X1].unwrap_or(0);
  lr.r[2] = regs[Aarch64::X2].unwrap_or(0);
  lr.r[3] = regs[Aarch64::X3].unwrap_or(0);
  lr.r[4] = regs[Aarch64::X4].unwrap_or(0);
  lr.r[5] = regs[Aarch64::X5].unwrap_or(0);
  lr.r[6] = regs[Aarch64::X6].unwrap_or(0);
  lr.r[7] = regs[Aarch64::X7].unwrap_or(0);
  lr.r[8] = regs[Aarch64::X8].unwrap_or(0);
  lr.r[9] = regs[Aarch64::X9].unwrap_or(0);
  lr.r[10] = regs[Aarch64::X10].unwrap_or(0);
  lr.r[11] = regs[Aarch64::X11].unwrap_or(0);
  lr.r[12] = regs[Aarch64::X12].unwrap_or(0);
  lr.r[13] = regs[Aarch64::X13].unwrap_or(0);
  lr.r[14] = regs[Aarch64::X14].unwrap_or(0);
  lr.r[15] = regs[Aarch64::X15].unwrap_or(0);
  lr.r[16] = regs[Aarch64::X16].unwrap_or(0);
  lr.r[17] = regs[Aarch64::X17].unwrap_or(0);
  lr.r[18] = regs[Aarch64::X18].unwrap_or(0);
  lr.r[19] = regs[Aarch64::X19].unwrap_or(0);
  lr.r[20] = regs[Aarch64::X20].unwrap_or(0);
  lr.r[21] = regs[Aarch64::X21].unwrap_or(0);
  lr.r[22] = regs[Aarch64::X22].unwrap_or(0);
  lr.r[23] = regs[Aarch64::X23].unwrap_or(0);
  lr.r[24] = regs[Aarch64::X24].unwrap_or(0);
  lr.r[25] = regs[Aarch64::X25].unwrap_or(0);
  lr.r[26] = regs[Aarch64::X26].unwrap_or(0);
  lr.r[27] = regs[Aarch64::X27].unwrap_or(0);
  lr.r[28] = regs[Aarch64::X28].unwrap_or(0);

  // for (i, v) in lr.vector_half.iter_mut().enumerate() {
  //   *v = regs[DwarfRegisterAArch64::V0 as u8 + i as u8].unwrap_or(0);
  // }
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
  let got = elf::section_by_name(".got").unwrap().start;
  let base_addrs = BaseAddresses::default()
    .set_eh_frame_hdr(eh_frame_hdr as u64)
    .set_eh_frame(eh_frame as u64)
    .set_text(text as u64)
    .set_got(got as u64);

  (unsafe { core::slice::from_raw_parts(
    ehf.start as usize as *const u8,
    (ehf.end - ehf.start) as usize
  ) }, base_addrs)
}

pub fn start_unwinding(stack_frames_to_skip: usize) -> Result<(), &'static str> {
  let unwinding_context_ptr = {
    Box::into_raw(
      Box::new(
        UnwindingContext {
          stack_frame_iter: StackFrameIter::new(Registers::default())
        }
      )
    )
  };

  let res = invoke_with_current_registers(|registers| {
    let unwinding_context = unsafe {
      &mut *unwinding_context_ptr
    };
    unwinding_context.stack_frame_iter.registers = registers;
    for _i in 0..stack_frames_to_skip {
      unwinding_context.stack_frame_iter.next()
        .map_err(|_e| {
          error!("error skipping call stack frame {} in unwinder", _i);
          "error skipping call stack frame in unwinder"
        })?
        .ok_or("call stack frame did not exist (we were trying to skip it)")?;
    }

    continue_unwinding(unwinding_context_ptr)
  });

  match &res {
    &Ok(()) => {
      debug!("unwinding procedure has reached the end of the stack.");
    }
    &Err(e) => {
      error!("BUG: unwinding the first stack frame returned unexpectedly. Error: {}", e);
    }
  }

  cleanup_unwinding_context(unwinding_context_ptr);
}

fn continue_unwinding(unwinding_context_ptr: *mut UnwindingContext) -> Result<(), &'static str> {
  let stack_frame_iter = unsafe { &mut (*unwinding_context_ptr).stack_frame_iter };

  trace!("continue_unwinding(): stack_frame_iter: {:#X?}", stack_frame_iter);

  let (mut regs, landing_pad_address) = if let Some(frame) = stack_frame_iter.next().map_err(|e| {
    error!("continue_unwinding: error getting next stack frame in the call stack: {}", e);
    "continue_unwinding: error getting next stack frame in the call stack"
  })? {
    {
      info!("Unwinding StackFrame: {:#X?}", frame);
      // info!("  Regs: {:#X?}", stack_frame_iter.registers);
    }

    if let Some(lsda) = frame.lsda {
      let lsda = lsda as usize;
      if let Some(lsda_slice) = elf::section_by_addr(lsda) {
        let table = lsda::GccExceptTableArea::new(lsda_slice, gimli::NativeEndian, frame.initial_address);

        // {
        //     let mut iter = table.call_site_table_entries().map_err(|_| "BAD TABLE")?;
        //     while let Some(entry) = iter.next().map_err(|_| "BAD ITER")? {
        //         debug!("    {:#X?}", entry);
        //     }
        // }

        let entry = match table.call_site_table_entry_for_address(frame.call_site_address) {
          Ok(x) => x,
          Err(e) => {
            error!("continue_unwinding(): couldn't find a call site table entry for this stack frame's call site address {:#X}. Error: {}", frame.call_site_address, e);

            // Now we don't have an exact match. We try to use the previous
            let mut iter = table.call_site_table_entries().map_err(|_e| { "Couldn't find call_site_table_entries" })?;

            let mut closest_entry = None;
            while let Some(entry) = iter.next().map_err(|_e| { "Couldn't iterate through the entries" })? {
              if entry.range_of_covered_addresses().start < frame.call_site_address {
                closest_entry = Some(entry);
              }
            }

            if let Some(closest_entry) = closest_entry {
              debug!("No unwind info for address. Using the closeset");
              closest_entry
            } else {
              return Err("continue_unwinding(): couldn't find a call site table entry for this stack frame's call site address.");
            }
          }
        };


        info!("Found call site entry for address {:#X}: {:#X?}", frame.call_site_address, entry);
        (stack_frame_iter.registers.clone(), entry.landing_pad_address())
      } else {
        error!("  BUG: couldn't find LSDA section (.gcc_except_table) for LSDA address: {:#X}", lsda);
        return Err("BUG: couldn't find LSDA section (.gcc_except_table) for LSDA address specified in stack frame");
      }
    } else {
      info!("continue_unwinding(): stack frame has no LSDA");
      return continue_unwinding(unwinding_context_ptr);
    }
  } else {
    info!("continue_unwinding(): NO REMAINING STACK FRAMES");
    return Ok(());
  };

  // Even if this frame has LSDA, it may still not have a landing pad function.
  let landing_pad_address = match landing_pad_address {
    Some(lpa) => lpa,
    _ => {
      warn!("continue_unwinding(): stack frame has LSDA but no landing pad");
      return continue_unwinding(unwinding_context_ptr);
    }
  };

  // Exception/interrupt handlers appear to have no real cleanup routines, despite having an LSDA entry. 
  // Thus, we skip unwinding an exception handler frame because its landing pad will point to an invalid instruction (usually `ud2`).
  if stack_frame_iter.last_frame_was_exception_handler {
    let landing_pad_value: u16 = unsafe { *(landing_pad_address as *const u16) };

    warn!("Skipping exception/interrupt handler's landing pad (cleanup function) at {:#X}, which points to {:#X} (UD2: {})",
          landing_pad_address, landing_pad_value, landing_pad_value == 0x0B0F,  // the `ud2` instruction
    );
    return continue_unwinding(unwinding_context_ptr);
  }

  unsafe {
    // brk #1
    if (landing_pad_address as usize as *const u32).read() == 0xd4_20_00_20 {
      error!("landing to {:#X} is `brk #1`", landing_pad_address);
      return continue_unwinding(unwinding_context_ptr);
    }
  }

  // Jump to the actual landing pad function, or rather, a function that will jump there after setting up register values properly.

  info!("Jumping to landing pad (cleanup function) at {:#X}", landing_pad_address);
  // Once the unwinding cleanup function is done, it will call _Unwind_Resume (technically, it jumps to it),
  // and pass the value in the landing registers' RAX register as the argument to _Unwind_Resume. 
  // So, whatever we put into RAX in the landing regs will be placed into the first arg (RDI) in _Unwind_Resume.
  // This is arch-specific; for x86_64 the transfer is from RAX -> RDI, for ARM/AARCH64, the transfer is from R0 -> R1 or X0 -> X1.
  // See this for more mappings: <https://github.com/rust-lang/rust/blob/master/src/libpanic_unwind/gcc.rs#L102>
  regs[Aarch64::X0] = Some(unwinding_context_ptr as u64);
  unsafe {
    land(&regs, landing_pad_address)?;
  }
  error!("BUG: call to unwind::land() returned, which should never happen!");
  Err("BUG: call to unwind::land() returned, which should never happen!")
}


/// This function should be invoked when the unwinding procedure is finished, or cannot be continued any further.
/// It cleans up the `UnwindingContext` object pointed to by the given pointer and marks the current task as killed.
fn cleanup_unwinding_context(unwinding_context_ptr: *mut UnwindingContext) -> ! {
  // Recover ownership of the unwinding context from its pointer
  let unwinding_context_boxed = unsafe { Box::from_raw(unwinding_context_ptr) };
  let unwinding_context = *unwinding_context_boxed;
  drop(unwinding_context.stack_frame_iter);
  loop {}
}

pub fn unwind_resume(unwinding_context_ptr: usize) -> ! {
  // trace!("unwind_resume(): unwinding_context_ptr value: {:#X}", unwinding_context_ptr);
  let unwinding_context_ptr = unwinding_context_ptr as *mut UnwindingContext;

  match continue_unwinding(unwinding_context_ptr) {
    Ok(()) => {
      debug!("unwind_resume(): continue_unwinding() returned Ok(), meaning it's at the end of the call stack.");
    }
    Err(e) => {
      error!("BUG: in unwind_resume(): continue_unwinding() returned an error: {}", e);
    }
  }
  // here, cleanup the unwinding state and kill the task
  cleanup_unwinding_context(unwinding_context_ptr);
}

