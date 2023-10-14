use rpabi::{CONFIG_USER_STACK_BTM, CONFIG_USER_STACK_TOP};

use crate::arch::PAGE_SIZE;
use crate::kernel::cpu::cpu;
use crate::kernel::traits::*;
use crate::mm::page_table::{PageTableEntryAttrTrait, PageTableTrait};
use crate::util::*;

pub fn handle() {
  let t = cpu().running_thread();
  match t {
    None => panic!(),
    Some(t) => match t.address_space() {
      None => panic!(),
      Some(a) => {
        let addr = crate::arch::Arch::fault_address();
        let va = round_down(addr, PAGE_SIZE);

        // NOTE: allocate stack region automatically
        if addr > CONFIG_USER_STACK_BTM && addr < CONFIG_USER_STACK_TOP {
          let pt = a.page_table();
          match pt.lookup_page(va) {
            None => {
              if let Ok(frame) = crate::mm::page_pool::page_alloc() {
                frame.zero();
                match pt.insert_page(va, crate::mm::Frame::from(frame),
                                     crate::mm::page_table::EntryAttribute::user_default()) {
                  Ok(_) => {
                    return;
                  }
                  Err(_) => {
                    warn!("stack page insert failed");
                  }
                }
              } else {
                warn!("stack page allocate oom");
              }
            }
            Some(_) => {
              // page already existed
            }
          }
        }
        let pt = a.page_table();
        info!("thread t{} asid {} page fault va {:x} pte {:X?} fall through", t.tid(), a.asid(), addr, pt.lookup_page(va));

        // default to user exception handler
        crate::kernel::exception::handle_user();
      }
    }
  }
}