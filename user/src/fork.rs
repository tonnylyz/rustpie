use crate::arch::page_table::*;
use crate::config::*;
use crate::syscall::*;

fn duplicate_page(asid: u16, va: usize, pte: EntryAttribute) {
  if pte.shared {
    mem_map(0, va, asid, va, pte);
  } else if pte.writable && !pte.copy_on_write {
    mem_map(0, va, asid, va, pte - PTE_W + PTE_COW);
    mem_map(0, va, 0, va, pte - PTE_W + PTE_COW);
  } else {
    mem_map(0, va, asid, va, pte);
  }
}

extern "C" {
  fn asm_page_fault_handler() -> !;
}

pub fn fork() -> i32 {
  match address_space_alloc() {
    (asid, tid) => if asid == 0 {
      // set_self_ipc(get_asid(0));
      0
    } else {
      traverse(TRAVERSE_LIMIT, |va, attr| {
        duplicate_page(asid, va, attr)
      });
      mem_alloc(asid, EXCEPTION_STACK_TOP - PAGE_SIZE, PTE_W);
      event_handler(asid, asm_page_fault_handler as usize, EXCEPTION_STACK_TOP, 0);
      thread_set_status(tid, ThreadStatus::TsRunnable);
      asid as i32
    },
    (u16::MAX, u16::MAX) => {
      println!("address_space_alloc error");
      -1
    }
  }
}

pub fn test() {
  println!("fork test started pid {}", get_asid(0));
  let mut a = 0;
  let mut id = fork();
  if id == 0 {
    id = fork();
    if id == 0 {
      a += 3;
      loop {
        print!("{}", a);
      }
    }
    a += 2;
    loop {
      print!("{}", a);
    }
  }
  a += 1;
  loop {
    print!("{}", a);
  }
}
