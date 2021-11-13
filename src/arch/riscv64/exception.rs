use riscv::regs::*;
use tock_registers::interfaces::{Readable, Writeable};

use crate::lib::traits::*;
use crate::lib::interrupt::InterruptController;
use crate::arch::ContextFrame;

global_asm!(include_str!("exception.S"));

const INTERRUPT_SUPERVISOR_SOFTWARE: usize = 1;
const INTERRUPT_SUPERVISOR_TIMER: usize = 5;
const INTERRUPT_SUPERVISOR_EXTERNAL: usize = 9;

const EXCEPTION_INSTRUCTION_ADDRESS_MISALIGNED: usize = 0;
const EXCEPTION_INSTRUCTION_ACCESS_FAULT: usize = 1;
const EXCEPTION_ILLEGAL_INSTRUCTION: usize = 2;
const EXCEPTION_BREAKPOINT: usize = 3;
const EXCEPTION_LOAD_ADDRESS_MISALIGNED: usize = 4;
const EXCEPTION_LOAD_ACCESS_FAULT: usize = 5;
const EXCEPTION_STORE_ADDRESS_MISALIGNED: usize = 6;
const EXCEPTION_STORE_ACCESS_FAULT: usize = 7;
const EXCEPTION_ENVIRONMENT_CALL_FROM_USER_MODE: usize = 8;
// const EXCEPTION_ENVIRONMENT_CALL_FROM_SUPERVISOR_MODE: usize = 9;
// const EXCEPTION_ENVIRONMENT_CALL_FROM_MACHINE_MODE: usize = 11;
const EXCEPTION_INSTRUCTION_PAGE_FAULT: usize = 12;
const EXCEPTION_LOAD_PAGE_FAULT: usize = 13;
const EXCEPTION_STORE_PAGE_FAULT: usize = 15;

#[no_mangle]
unsafe extern "C" fn exception_entry(ctx: *mut ContextFrame) {
  let from_kernel = SSTATUS.is_set(SSTATUS::SPP);
  let core = crate::lib::cpu::cpu();
  core.set_context(ctx);
  let cause = SCAUSE.get();
  let irq = (cause >> 63) != 0;
  let code = (cause & 0xf) as usize;
  if from_kernel && !irq {
    info!("SCAUSE {:016x}", cause);
    info!("SEPC {:016x}", core.context().exception_pc());
    info!("FAR  {:016x}", crate::arch::Arch::fault_address());
    match code {
      EXCEPTION_INSTRUCTION_ADDRESS_MISALIGNED
      | EXCEPTION_INSTRUCTION_ACCESS_FAULT
      | EXCEPTION_ILLEGAL_INSTRUCTION
      | EXCEPTION_BREAKPOINT
      | EXCEPTION_LOAD_ADDRESS_MISALIGNED
      | EXCEPTION_LOAD_ACCESS_FAULT
      | EXCEPTION_STORE_ADDRESS_MISALIGNED
      | EXCEPTION_STORE_ACCESS_FAULT => crate::lib::exception::handle_kernel(ctx.as_ref().unwrap(), false),
      EXCEPTION_INSTRUCTION_PAGE_FAULT
      | EXCEPTION_LOAD_PAGE_FAULT
      | EXCEPTION_STORE_PAGE_FAULT => crate::lib::exception::handle_kernel(ctx.as_ref().unwrap(), true),
      _ => panic!("unhandled kernel exception"),
    }
  }
  if irq {
    match code {
      INTERRUPT_SUPERVISOR_SOFTWARE => panic!("Interrupt::SupervisorSoft"),
      INTERRUPT_SUPERVISOR_TIMER => crate::lib::timer::interrupt(),
      INTERRUPT_SUPERVISOR_EXTERNAL => {
        let plic = &crate::driver::INTERRUPT_CONTROLLER;
        if let Some(int) = plic.fetch() {
          crate::lib::interrupt::interrupt(int);
          plic.finish(int);
        } else {
          warn!("PLIC report no irq");
        }
      }
      _ => panic!("Interrupt::Unknown"),
    }
  } else {
    match code {
      EXCEPTION_INSTRUCTION_ADDRESS_MISALIGNED
      | EXCEPTION_INSTRUCTION_ACCESS_FAULT
      | EXCEPTION_ILLEGAL_INSTRUCTION
      | EXCEPTION_BREAKPOINT
      | EXCEPTION_LOAD_ADDRESS_MISALIGNED
      | EXCEPTION_STORE_ADDRESS_MISALIGNED => {
        info!("SCAUSE {:016x}", cause);
        info!("SEPC {:016x}", core.context().exception_pc());
        info!("FAR  {:016x}", crate::arch::Arch::fault_address());
        crate::lib::exception::handle_user()
      },
      EXCEPTION_ENVIRONMENT_CALL_FROM_USER_MODE => {
        // Note: we need to set epc to next instruction before doing system call
        //       pay attention to yield and process_alloc
        let pc = core.context_mut().exception_pc();
        core.context_mut().set_exception_pc(pc + 4);
        crate::lib::syscall::syscall();
      }
      EXCEPTION_INSTRUCTION_PAGE_FAULT
      | EXCEPTION_LOAD_ACCESS_FAULT
      | EXCEPTION_STORE_ACCESS_FAULT
      | EXCEPTION_LOAD_PAGE_FAULT
      | EXCEPTION_STORE_PAGE_FAULT => {
        crate::mm::page_fault::handle()
      },
      _ => panic!("Exception::Unknown")
    }
  }
  core.clear_context();
}

pub fn init() {
  extern "C" {
    fn push_context();
  }
  STVEC.write(STVEC::BASE.val(push_context as usize as u64 >> 2) + STVEC::MODE::Direct);
  // Note: riscv vector only 4 byte per cause
  //       direct mode make it distributed later in `exception_entry`
  SIE.write(SIE::SEIE::SET);
}
