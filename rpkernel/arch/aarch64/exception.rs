use core::mem::size_of;
use aarch64_cpu::registers::{ESR_EL1, VBAR_EL1};
use tock_registers::interfaces::{Readable, Writeable};

use crate::arch::ContextFrame;
use crate::kernel::traits::ContextFrameTrait;

core::arch::global_asm!(include_str!("exception.S"));

#[no_mangle]
unsafe extern "C" fn current_el_sp0_synchronous(ctx: *mut ContextFrame) {
  panic!("current_el_sp0_synchronous\n{}", ctx.read());
}

#[no_mangle]
unsafe extern "C" fn current_el_sp0_irq(ctx: *mut ContextFrame) {
  // panic!("current_el_sp0_irq\n{}", ctx.read());
  lower_aarch64_irq(ctx);
}

#[no_mangle]
unsafe extern "C" fn current_el_spx_synchronous(ctx: *mut ContextFrame) {
  let ec = ESR_EL1.read(ESR_EL1::EC);
  error!("current_el_spx_synchronous EC {:#X} \n{}", ec, ctx.read());
  let ctx_mut = ctx.as_mut().unwrap();
  ctx_mut.set_stack_pointer(ctx as usize + size_of::<ContextFrame>());
  let page_fault = ESR_EL1.matches_all(ESR_EL1::EC::InstrAbortCurrentEL) | ESR_EL1.matches_all(ESR_EL1::EC::DataAbortCurrentEL);
  crate::kernel::exception::handle_kernel(ctx.as_ref().unwrap(), page_fault);
  loop {}
}

#[no_mangle]
unsafe extern "C" fn current_el_spx_irq(ctx: *mut ContextFrame) {
  panic!("current_el_spx_irq\n{}", ctx.read());
}

#[no_mangle]
unsafe extern "C" fn current_el_spx_serror(ctx: *mut ContextFrame) {
  panic!("current_el_spx_serror\n{}", ctx.read());
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_synchronous(ctx: *mut ContextFrame) {
  let core = crate::kernel::cpu::cpu();
  core.set_context(ctx);
  if ESR_EL1.matches_all(ESR_EL1::EC::SVC64) {
    crate::kernel::syscall::syscall();
  } else if ESR_EL1.matches_all(ESR_EL1::EC::InstrAbortLowerEL) | ESR_EL1.matches_all(ESR_EL1::EC::DataAbortLowerEL) {
    crate::mm::page_fault::handle();
  } else {
    let ec = ESR_EL1.read(ESR_EL1::EC);
    error!("lower_aarch64_synchronous: ec {:06b} \n{}", ec, ctx.read());
    crate::kernel::exception::handle_user();
  }
  core.clear_context();
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_irq(ctx: *mut ContextFrame) {
  use crate::kernel::interrupt::*;
  let core = crate::kernel::cpu::cpu();
  core.set_context(ctx);
  use crate::driver::{INTERRUPT_CONTROLLER, gic::INT_TIMER};
  let irq = INTERRUPT_CONTROLLER.fetch();
  match irq {
    Some((INT_TIMER, 0)) => {
      crate::kernel::timer::interrupt();
    }
    Some((int_id, src_cpu)) => {
      if int_id >= 32 {
        crate::kernel::interrupt::interrupt(int_id);
      } else if int_id < 16 {
        crate::kernel::interrupt::ipi_interrupt(int_id.into(), src_cpu);
      } else {
        panic!("GIC unhandled PPI")
      }
    }
    None => {
      warn!("GIC special irq")
    }
  }
  if irq.is_some() {
    INTERRUPT_CONTROLLER.finish(irq.unwrap().0);
  }
  core.clear_context();
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_serror(ctx: *mut ContextFrame) {
  let core = crate::kernel::cpu::cpu();
  core.set_context(ctx);
  crate::kernel::exception::handle_user();
  core.clear_context();
}

pub fn init() {
  extern "C" {
    fn vectors();
  }
  let addr: u64 = vectors as usize as u64;
  VBAR_EL1.set(addr);
  use aarch64_cpu::asm::barrier::*;
  isb(SY);
}
