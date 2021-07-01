use xmas_elf::*;
use xmas_elf::sections::SectionData;
use core::ops::Range;

extern "C" {
  static ELF_IMAGE: [u8; 0x40000000];
}

pub fn section_by_name(name: &'static str) -> Option<Range<u64>> {
  if let Ok(elf) = ElfFile::new(unsafe { &ELF_IMAGE }) {
    for section_header in elf.section_iter() {
      if let Ok(section_name) = section_header.get_name(&elf) {
        // println!("section {}", section_name);
        if section_name == name {
          return Some(section_header.address()..(section_header.address() + section_header.size()))
        }
      } else {
        continue
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
            }
          }
          Err(_) => {}
        }
      }

    }
  }
  None
}