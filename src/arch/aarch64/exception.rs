use cortex_a::{barrier, regs::*};

use crate::arch::ContextFrame;
use crate::lib::cpu::{CoreTrait, current};
use crate::core_id;

global_asm!(include_str!("exception.S"));

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
  panic!("current_el_spx_synchronous\n{}", ctx.read());
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
  let core = crate::lib::cpu::current();
  core.set_context(ctx);
  if ESR_EL1.matches_all(ESR_EL1::EC::SVC64) {
    crate::lib::syscall::syscall();
  } else if ESR_EL1.matches_all(ESR_EL1::EC::InstrAbortLowerEL) | ESR_EL1.matches_all(ESR_EL1::EC::DataAbortLowerEL) {
    crate::mm::page_fault::handle();
  } else {
    let ec = ESR_EL1.read(ESR_EL1::EC);
    error!("lower_aarch64_synchronous: ec {:06b} \n{}", ec, ctx.read());
    error!("thread t{} core{}", current().running_thread().unwrap().tid(), core_id());
    // if let Some(t) = current().running_thread() {
    //   info!("{:#x?}", t);
    // }
    crate::lib::exception::handle();
  }
  core.clear_context();
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_irq(ctx: *mut ContextFrame) {
  use crate::lib::interrupt::*;
  let core = crate::lib::cpu::current();
  core.set_context(ctx);
  use crate::driver::{INTERRUPT_CONTROLLER, gic::INT_TIMER};
  let irq = INTERRUPT_CONTROLLER.fetch();
  match irq {
    Some(INT_TIMER) => {
      crate::lib::timer::interrupt();
    }
    Some(i) => {
      if i >= 32 {
        crate::lib::interrupt::interrupt(i);
      } else {
        panic!("GIC unhandled SGI PPI")
      }
    }
    None => {
      panic!("GIC unknown irq")
    }
  }
  if irq.is_some() {
    INTERRUPT_CONTROLLER.finish(irq.unwrap());
  }
  core.clear_context();
}

#[no_mangle]
unsafe extern "C" fn lower_aarch64_serror(ctx: *mut ContextFrame) {
  let core = crate::lib::cpu::current();
  core.set_context(ctx);
  crate::lib::exception::handle();
  core.clear_context();
}

pub fn init() {
  extern "C" {
    fn vectors();
  }
  unsafe {
    let addr: u64 = vectors as usize as u64;
    VBAR_EL1.set(addr);
    barrier::isb(barrier::SY);
  }
}
