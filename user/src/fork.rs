use crate::arch::page_table::*;
use crate::config::*;
use crate::ipc::set_self_ipc;
use crate::syscall::*;

fn duplicate_page(pid: u16, va: usize, pte: EntryAttribute) {
  if pte.shared {
    match mem_map(0, va, pid, va, pte) {
      Ok(_) => {}
      Err(_) => { panic!("duplicate_page: mem_map failed") }
    }
  } else if pte.writable && !pte.copy_on_write {
    match mem_map(0, va, pid, va, pte - PTE_W + PTE_COW) {
      Ok(_) => {}
      Err(_) => { panic!("duplicate_page: mem_map failed") }
    }
    match mem_map(0, va, 0, va, pte - PTE_W + PTE_COW) {
      Ok(_) => {}
      Err(_) => { panic!("duplicate_page: mem_map failed") }
    }
  } else {
    match mem_map(0, va, pid, va, pte) {
      Ok(_) => {}
      Err(_) => { panic!("duplicate_page: mem_map failed") }
    }
  }
}

extern "C" {
  fn asm_page_fault_handler() -> !;
}

pub fn fork() -> i32 {
  match process_alloc() {
    Ok(pid) => if pid == 0 {
      set_self_ipc(getpid());
      0
    } else {
      traverse(TRAVERSE_LIMIT, |va, attr| {
        duplicate_page(pid, va, attr)
      });
      match mem_alloc(pid, EXCEPTION_STACK_TOP - PAGE_SIZE, PTE_W) {
        Ok(_) => {}
        Err(_) => { panic!("fork: mem_alloc failed") }
      }
      match process_set_exception_handler(pid, asm_page_fault_handler as usize, EXCEPTION_STACK_TOP) {
        Ok(_) => {}
        Err(_) => { panic!("fork: process_set_exception_handler failed") }
      }
      match process_set_status(pid, ProcessStatus::PsRunnable) {
        Ok(_) => {}
        Err(_) => { panic!("fork: process_set_status failed") }
      }
      pid as i32
    },
    Err(e) => {
      println!("process_alloc error {:?}", e);
      -1
    }
  }
}