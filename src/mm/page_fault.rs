use common::{CONFIG_USER_STACK_BTM, CONFIG_USER_STACK_TOP};

use crate::arch::PAGE_SIZE;
use crate::lib::cpu::cpu;
use crate::lib::traits::*;
use crate::util::*;
use crate::mm::page_table::{PageTableTrait, PageTableEntryAttrTrait};

pub fn handle() {
  let t = cpu().running_thread();
  match t {
    None => {
      crate::lib::cpu::cpu().schedule();
      return;
    }
    Some(t) => match t.address_space() {
      None => {
        crate::lib::cpu::cpu().schedule();
        return;
      }
      Some(a) => {

        let addr = crate::arch::Arch::fault_address();
        let va = round_down(addr, PAGE_SIZE);
        trace!("thread t{} core {} page fault {:x}", t.tid(), crate::arch::Arch::core_id(), va);

        // NOTE: allocate stack region automatically
        if addr > CONFIG_USER_STACK_BTM && addr < CONFIG_USER_STACK_TOP {
          let pt = a.page_table();
          match pt.lookup_page(va) {
            None => {
              if let Ok(frame) = crate::mm::page_pool::page_alloc() {
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
        // default to user exception handler
        crate::lib::exception::handle_user();
      }
    }
  }
}