#![no_std]
#![feature(global_asm)]
#![feature(trait_alias)]
#![feature(core_intrinsics)]

extern crate alloc;
#[macro_use]
extern crate log;

use alloc::boxed::Box;

use fallible_iterator::FallibleIterator;
use gimli::{CfaRule, Pointer, UninitializedUnwindContext, UnwindSection, read::RegisterRule};

use registers::Registers;

#[cfg(target_arch = "aarch64")]
#[path = "arch/aarch64.rs"]
pub mod arch;

#[cfg(target_arch = "riscv64")]
#[path = "arch/riscv64.rs"]
pub mod arch;

use arch::*;


pub mod registers;

pub mod elf;
mod lsda;
pub mod catch;

pub struct UnwindingContext {
  skip: usize,
  stack_frame_iter: StackFrameIter,
}

pub struct StackFrame {
  lsda: Option<u64>,
  initial_address: u64,
  call_site_address: u64,
}

pub struct StackFrameIter {
  registers: Registers,
  /// State: (Caller, CFA)
  state: Option<(u64, u64)>,
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
    if let Some((caller, cfa)) = self.state.take() {
      let mut new_regs = registers.clone();
      new_regs[REG_RETURN_ADDRESS] = None;
      new_regs[REG_STACK_POINTER] = Some(cfa);
      let base_addrs = elf::base_addresses();
      let eh_frame = gimli::read::EhFrame::new(
        elf::eh_frame_slice(),
        gimli::NativeEndian,
      );
      let mut unwind_ctx = UninitializedUnwindContext::new();
      let fde = eh_frame.fde_for_address(&base_addrs, caller, gimli::EhFrame::cie_from_offset).map_err(|_e| {
        error!("gimli error: {:?}", _e);
        "gimli error while finding FDE for address"
      })?;
      let row = fde.unwind_info_for_address(&eh_frame, &base_addrs, &mut unwind_ctx, caller).map_err(|_e| {
        error!("gimli error: {:?}", _e);
        "gimli error while finding unwind info for address"
      })?;
      for &(reg_num, ref rule) in row.registers() {
        if reg_num == REG_STACK_POINTER {
          continue;
        }
        new_regs[reg_num] = match *rule {
          RegisterRule::Undefined
          | RegisterRule::Expression(_)
          | RegisterRule::ValExpression(_)
          | RegisterRule::Architectural => {
            error!("unsupported rule {:?}", *rule);
            return Err("unsupported rule");
          }
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
      *registers = new_regs;
    }
    let return_address = match registers[REG_RETURN_ADDRESS] {
      Some(0) | None => return Ok(None),
      Some(ra) => ra,
    };
    let caller = return_address - 4;
    let base_addrs = elf::base_addresses();
    let eh_frame = gimli::read::EhFrame::new(
      elf::eh_frame_slice(),
      gimli::NativeEndian,
    );
    let mut unwind_ctx = UninitializedUnwindContext::new();
    let fde = eh_frame.fde_for_address(&base_addrs, caller, gimli::EhFrame::cie_from_offset).map_err(|_e| {
      error!("gimli error: {:?}", _e);
      "gimli error while finding FDE for address"
    })?;
    let row = fde.unwind_info_for_address(&eh_frame, &base_addrs, &mut unwind_ctx, caller).map_err(|_e| {
      error!("gimli error: {:?}", _e);
      "gimli error while finding unwind info for address"
    })?;

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
      lsda: fde.lsda().map(|x| unsafe {
        match x {
          Pointer::Direct(x) => x,
          Pointer::Indirect(x) => *(x as *const u64),
        }
      }),
      initial_address: fde.initial_address(),
      call_site_address: caller,
    };

    self.state = Some((caller, cfa));
    Ok(Some(frame))
  }
}

extern "C" {
  #[allow(improper_ctypes)]
  fn unwind_trampoline(ctx: usize);
}

pub fn unwind_from_exception(registers: Registers) -> ! {
  let ctx = Box::into_raw(Box::new(UnwindingContext {
    skip: 0,
    stack_frame_iter: StackFrameIter::new(registers)
  }));
  unwind(ctx);
  cleanup(ctx);
  error!("unwind failed!");
  loop {}
}

pub fn unwind_from_panic(stack_frames_to_skip: usize) -> ! {
  let ctx = Box::into_raw(Box::new(UnwindingContext {
    skip: stack_frames_to_skip,
    stack_frame_iter: StackFrameIter::new(Registers::default())
  }));
  unsafe {
    unwind_trampoline(ctx as usize);
  }
  cleanup(ctx);
  error!("unwind failed!");
  loop {}
}

fn unwind_from_panic_stub(registers: Registers, ctx2: *mut UnwindingContext) {
  let ctx = unsafe { &mut *ctx2 };
  ctx.stack_frame_iter.registers = registers;
  for _i in 0..ctx.skip {
    let _ = ctx.stack_frame_iter.next();
  }
  unwind(ctx2);
}

/// Main unwind function
/// Only return when unwind failed, otherwise jump to a landing pad
fn unwind(ctx: *mut UnwindingContext) {
  let stack_frame_iter = unsafe { &mut (*ctx).stack_frame_iter };
  match stack_frame_iter.next() {
    Err(e) => {
      error!("next failed with {}", e);
      return;
    }
    Ok(None) => {
      error!("no frame left");
      return;
    },
    Ok(Some(frame)) => {
      info!("function addr {:016x}", frame.initial_address);
      info!("call site {:016x}", frame.call_site_address);
      match frame.lsda {
        None => {
          warn!("frame has no lsda, continue");
          return unwind(ctx);
        }
        Some(lsda) => {
          match elf::section_by_addr(lsda as usize) {
            None => {
              error!("cannot find lsda in elf");
              return;
            }
            Some(lsda_slice) => {
              let table = lsda::GccExceptTableArea::new(lsda_slice, gimli::NativeEndian, frame.initial_address);
              let entry = match table.call_site_table_entry_for_address(frame.call_site_address) {
                Ok(x) => x,
                Err(e) => {
                  warn!("call site has no entry {:016x} {}", frame.call_site_address, e);
                  let mut iter = match table.call_site_table_entries() {
                    Ok(iter) => iter,
                    Err(_) => {
                      error!("call_site_table_entries failed");
                      return;
                    }
                  };
                  let mut closest_entry = None;
                  loop {
                    let next = match iter.next() {
                      Ok(next) => next,
                      Err(_) => {
                        error!("iter call site table failed");
                        return;
                      }
                    };
                    if let Some(entry) = next {
                      if entry.range_of_covered_addresses().start < frame.call_site_address {
                        closest_entry = Some(entry);
                      }
                    } else {
                      break;
                    }
                  }
                  if let Some(closest_entry) = closest_entry {
                    closest_entry
                  } else {
                    error!("no closest entry");
                    return;
                  }
                }
              };
              let landing_pad = match entry.landing_pad_address() {
                Some(lpa) => lpa,
                _ => {
                  warn!("frame has lsda but no landing pad, continue");
                  return unwind(ctx);
                }
              };

              #[cfg(target_arch = "aarch64")]
              unsafe {
                // brk #?
                if (landing_pad as usize as *const u32).read() & 0xFF_E0_00_00 == 0xd4_20_00_00 {
                  warn!("land at {:016x} is `brk #?`, continue", landing_pad);
                  return unwind(ctx);
                }
              }
              info!("land at {:016x}", landing_pad);
              let mut regs = stack_frame_iter.registers.clone();
              regs[REG_ARGUMENT] = Some(ctx as u64);
              unsafe {
                land(&regs, landing_pad);
              }
              // Never return!
            }
          }
        }
      }
    }
  };
}

/// Drop `UnwindingContext` when unwind fails
fn cleanup(ctx: *mut UnwindingContext) {
  let ctx = unsafe { Box::from_raw(ctx) };
  drop(ctx);
}

/// This function is called by `_Unwind_Resume`
///
/// An unwind process might land on multiple landing pads.
/// In the end of in-betweeen landing pad, _Unwind_Resume will be called to resume unwinding.
///
/// # Argument
///
/// * `arg` - pointer to `UnwindingContext`
pub fn unwind_resume(arg: usize) -> ! {
  let ctx = arg as *mut UnwindingContext;
  unwind(ctx);
  cleanup(ctx);
  error!("unwind failed!");
  loop {}
}
