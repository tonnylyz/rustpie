pub use self::cache::DiskCache;
pub use self::virtio_client::VirtioClient;

mod cache;
mod virtio_client;

use libtrusted::redoxcall::*;

/// A disk
pub trait Disk {
  fn read_at(&mut self, block: u64, buffer: &mut [u8]) -> Result<usize>;
  fn write_at(&mut self, block: u64, buffer: &[u8]) -> Result<usize>;
  fn size(&mut self) -> Result<u64>;
}
