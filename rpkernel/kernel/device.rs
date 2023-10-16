use crate::arch::PAGE_SIZE;
use crate::driver::Interrupt;
use crate::mm::Frame;
use crate::util::round_down;
use alloc::vec::Vec;
use core::ops::Range;
use core::panic;
use fdt::Fdt;
use fdt::node::FdtNode;

pub const DEVICE_NAME_LEN: usize = 32;
#[derive(Debug)]
pub struct Device {
  pub name: [u8; DEVICE_NAME_LEN],
  pub register: Range<usize>,
  pub interrupt: Option<Interrupt>,
}

pub const PLATFORM_DEVICE_LEN: usize = 8;

#[repr(align(4096))]
#[derive(Default)]
pub struct PlatformInfo {
  pub devices: [Option<Device>; PLATFORM_DEVICE_LEN],
}

impl Device {
  #[allow(dead_code)]
  pub fn new(name: &'static str, register: Range<usize>, interrupt: Option<Interrupt>) -> Self {
    let mut r = Device {
      name: [0; DEVICE_NAME_LEN],
      register,
      interrupt,
    };
    let bytes = name.as_bytes();
    for i in 0..bytes.len() {
      r.name[i] = bytes[i];
    }
    r
  }

  pub fn from_fdt_node(fdt: &Fdt, node: &FdtNode) -> Self {
    let mut regs = node.reg().unwrap();
    let register;
    let interrupt;
    let reg = regs.next().unwrap();
    let start = reg.starting_address as usize;
    register = start..(start + reg.size.unwrap());
    let intc = match node.interrupt_parent() {
        Some(node) => node,
        None => match fdt.find_node("/").unwrap().interrupt_parent() {
            Some(node) => node,
            None => panic!(),
        },
    };
    match intc.interrupt_cells() {
      Some(3) => {
        // GIC
        interrupt = match node.property("interrupts") {
            Some(prop) => match prop.as_triple_u32() {
                Some((irq_type, irq_num, _trigger_type)) => {
                  assert_eq!(irq_type, 0); // only spi type is allowed
                  Some(irq_num as usize + 32)
                },
                None => None,
            },
            None => None,
        }
      }
      Some(1) => {
        // PLIC
        if let Some(mut iter) = node.interrupts() {
          let irq_num = iter.next().unwrap();
          interrupt = Some(irq_num);
        } else {
          interrupt = None;
        }
      }
      _ => panic!(),
    }
    let mut r = Device {
      name: [0; DEVICE_NAME_LEN],
      register: register,
      interrupt: interrupt,
    };
    let bytes = node.name.as_bytes();
    for i in 0..bytes.len() {
      r.name[i] = bytes[i];
    }
    r
  }

  pub fn to_user_frames(&self) -> Vec<Frame> {
    let mut result = Vec::new();
    let start = round_down(self.register.start, PAGE_SIZE);
    for pa in (start..self.register.end).step_by(PAGE_SIZE) {
      result.push(Frame::from(pa));
    }
    result
  }
}
