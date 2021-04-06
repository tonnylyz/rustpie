use core::mem::size_of;

use crate::arch::{Address, ArchTrait, ContextFrame, ContextFrameTrait, PAGE_SIZE};
use crate::config::CONFIG_USER_LIMIT;
use crate::driver::{Interrupt};
use crate::lib::core::{CoreTrait, current};
use crate::lib::event::Event;
use crate::lib::page_table::PageTableTrait;
use crate::lib::round_down;

pub trait InterruptServiceRoutine {
  fn system_call();
  fn timer_interrupt();
  fn external_interrupt(int: Interrupt);
  fn page_fault();
  fn default();
}

pub struct Isr;

impl InterruptServiceRoutine for Isr {
  fn system_call() {
    crate::lib::syscall::syscall();
  }

  fn timer_interrupt() {
    crate::driver::timer::next();
    crate::lib::core::current().schedule();
  }

  fn external_interrupt(int: Interrupt) {
    println!("external_interrupt {}", int);
    match crate::lib::interrupt::INTERRUPT_WAIT.get(int) {
      None => { println!("irq not registered"); }
      Some(t) => {
        match t.address_space() {
          None => { panic!("kernel thread interrupt?") }
          Some(a) => {
            match a.event_handler(Event::Interrupt(int)) {
              None => { println!("no event handler") }
              Some((pc, sp)) => {
                let nt = crate::lib::thread::new_user(pc, sp, int, a.clone(), None);
                nt.set_status(crate::lib::thread::Status::TsRunnable);

                crate::driver::timer::next();
                crate::lib::core::current().schedule();
              }
            }
          }
        }
      }
    }
  }

  fn page_fault() {
    let t = crate::lib::core::current().running_thread();
    if t.is_none() {
      panic!("isr: page_fault: no running thread");
    }
    let t = t.unwrap();
    if t.address_space().is_none() {
      panic!("isr: kernel thread page fault");
    }
    let p = t.address_space().unwrap();

    let addr = crate::arch::Arch::fault_address();
    let va = round_down(addr, PAGE_SIZE);
    if va >= CONFIG_USER_LIMIT {
      println!("isr: page_fault: va >= CONFIG_USER_LIMIT, process killed");
      return;
    }
    if p.event_handler(Event::PageFault).is_none() {
      println!("isr: page_fault: {:016x}", addr);
      println!("isr: page_fault: process has no handler, process killed");
      return;
    }
    let (entry, stack_top) = p.event_handler(Event::PageFault).unwrap();
    let page_table = p.page_table();
    let stack_btm = stack_top - PAGE_SIZE;
    match page_table.lookup_user_page(stack_btm) {
      Some(uf) => {
        if va == stack_btm {
          println!("isr: page_fault: fault on exception stack, process killed");
          return;
        }
        let ctx = current().context_mut();
        unsafe {
          ((uf.pa().pa2kva() + PAGE_SIZE - size_of::<ContextFrame>()) as *mut ContextFrame)
            .write_volatile(*ctx);
        }
        ctx.set_exception_pc(entry);
        ctx.set_stack_pointer(stack_top - size_of::<ContextFrame>());
        ctx.set_argument(va);
        return;
      }
      None => {
        println!("isr: page_fault: exception stack not valid, process killed");
        return;
      }
    }
  }

  fn default() {
    match crate::lib::core::current().running_thread() {
      None => { panic!("isr: default: no running thread") }
      Some(t) => {
        t.destroy();
      }
    }
  }
}
