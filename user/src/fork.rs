use crate::arch::page_table::*;
use crate::config::*;
use crate::ipc::set_self_ipc;
use crate::syscall::*;
use rlibc::memcpy;

fn duplicate_page(asid: u16, va: usize, pte: EntryAttribute) {
  if pte.shared {
    match mem_map(0, va, asid, va, pte) {
      Ok(_) => {}
      Err(_) => { panic!("duplicate_page: mem_map failed") }
    }
  } else if pte.writable /*&& !pte.copy_on_write*/ {
    mem_alloc(asid, va, pte);
    mem_map(asid, va, 0, 0x1000_0000, pte);
    unsafe { memcpy(0x1000_0000 as *mut u8, va as *mut u8, PAGE_SIZE); }
    mem_unmap(0, 0x1000_0000);

    // match mem_map(0, va, asid, va, pte - PTE_W + PTE_COW) {
    //   Ok(_) => {}
    //   Err(_) => { panic!("duplicate_page: mem_map failed") }
    // }
    // match mem_map(0, va, 0, va, pte - PTE_W + PTE_COW) {
    //   Ok(_) => {}
    //   Err(_) => { panic!("duplicate_page: mem_map failed") }
    // }
  } else {
    match mem_map(0, va, asid, va, pte) {
      Ok(_) => {}
      Err(_) => { panic!("duplicate_page: mem_map failed") }
    }
  }
}

extern "C" {
  fn asm_page_fault_handler() -> !;
}

pub fn fork() -> i32 {
  match address_space_alloc() {
    Ok((asid, tid)) => if asid == 0 {
      set_self_ipc(get_asid());
      0
    } else {
      traverse(TRAVERSE_LIMIT, |va, attr| {
        duplicate_page(asid, va, attr)
      });
      // match mem_alloc(asid, EXCEPTION_STACK_TOP - PAGE_SIZE, PTE_W) {
      //   Ok(_) => {}
      //   Err(_) => { panic!("fork: mem_alloc failed") }
      // }
      // match process_set_exception_handler(asid, asm_page_fault_handler as usize, EXCEPTION_STACK_TOP) {
      //   Ok(_) => {}
      //   Err(_) => { panic!("fork: process_set_exception_handler failed") }
      // }
      match thread_set_status(tid, ThreadStatus::TsRunnable) {
        Ok(_) => {}
        Err(_) => { panic!("fork: process_set_status failed") }
      }
      asid as i32
    },
    Err(e) => {
      println!("process_alloc error {:?}", e);
      -1
    }
  }
}