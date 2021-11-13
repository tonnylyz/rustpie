use crate::fs::{Extent, BLOCK_SIZE};
use core::{fmt, ops, slice, mem};
use alloc::vec::Vec;

// An extra node
#[repr(packed)]
pub struct ExNode {
    pub prev: u64,
    pub next: u64,
    pub extents: [Extent; (BLOCK_SIZE as usize - 16) / 16],
}

impl ExNode {
    #[allow(dead_code)]
    pub fn default() -> ExNode {
        ExNode {
            prev: 0,
            next: 0,
            extents: [Extent::default(); (BLOCK_SIZE as usize - 16) / 16],
        }
    }
    #[allow(dead_code)]
    pub fn size(&self) -> u64 {
        self.extents
            .iter()
            .fold(0, |size, extent| size + extent.length)
    }
}

impl fmt::Debug for ExNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let extents: Vec<&Extent> = self
            .extents
            .iter()
            .filter(|extent| -> bool { extent.length > 0 })
            .collect();
        #[allow(unused_unsafe)]
        unsafe {
            #[allow(unaligned_references)]
            f.debug_struct("ExNode")
                .field("prev", &self.prev)
                .field("next", &self.next)
                .field("extents", &extents)
                .finish()
        }
    }
}

impl ops::Deref for ExNode {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(self as *const ExNode as *const u8, mem::size_of::<ExNode>())
                as &[u8]
        }
    }
}

impl ops::DerefMut for ExNode {
    fn deref_mut(&mut self) -> &mut [u8] {
        unsafe {
            slice::from_raw_parts_mut(self as *mut ExNode as *mut u8, mem::size_of::<ExNode>())
                as &mut [u8]
        }
    }
}

#[test]
fn ex_node_size_test() {
    assert_eq!(mem::size_of::<ExNode>(), BLOCK_SIZE as usize);
}
