use spin::Once;
use xmas_elf::*;
use xmas_elf::sections::SectionData;
use core::ops::Range;
use gimli::BaseAddresses;

extern "C" {
  static ELF_IMAGE: [u8; 0x40000000];
}

static BASE_ADDRESSES: Once<BaseAddresses> = Once::new();

pub fn base_addresses() -> BaseAddresses {
  match BASE_ADDRESSES.get() {
    Some(r) => r,
    None => {
      BASE_ADDRESSES.call_once(|| {
        let eh_frame = section_by_name(".eh_frame").unwrap();
        let eh_frame_hdr = section_by_name(".eh_frame_hdr").unwrap();
        let text = section_by_name(".text").unwrap();
        let result = BaseAddresses::default()
          .set_eh_frame_hdr(eh_frame_hdr.start)
          .set_eh_frame(eh_frame.start)
          .set_text(text.start);
        result
      })
    }
  }.clone()
}

static EH_FRAME: Once<Range<u64>> = Once::new();

fn eh_frame() -> Range<u64> {
  match EH_FRAME.get() {
    Some(r) => r,
    None => {
      EH_FRAME.call_once(|| {
        section_by_name(".eh_frame").unwrap()
      })
    }
  }.clone()
}

pub fn eh_frame_slice() -> &'static [u8] {
  let eh_frame = eh_frame();
  unsafe {
    core::slice::from_raw_parts(
      eh_frame.start as usize as *const u8,
      (eh_frame.end - eh_frame.start) as usize,
    )
  }
}

fn section_by_name(name: &'static str) -> Option<Range<u64>> {
  if let Ok(elf) = ElfFile::new(unsafe { &ELF_IMAGE }) {
    for section_header in elf.section_iter() {
      if let Ok(section_name) = section_header.get_name(&elf) {
        if section_name == name {
          return Some(section_header.address()..(section_header.address() + section_header.size()));
        }
      } else {
        continue;
      }
    }
  }
  None
}

pub fn section_by_addr(addr: usize) -> Option<&'static [u8]> {
  if let Ok(elf) = ElfFile::new(unsafe { &ELF_IMAGE }) {
    for section_header in elf.section_iter() {
      if addr >= section_header.address() as usize && addr < (section_header.address() + section_header.size()) as usize {
        match section_header.get_data(&elf) {
          Ok(x) => {
            return match x {
              SectionData::Undefined(r) => {
                Some(r)
              }
              _ => {
                None
              }
            };
          }
          Err(_) => {}
        }
      }
    }
  }
  None
}