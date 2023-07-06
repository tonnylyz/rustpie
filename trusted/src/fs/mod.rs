pub use mount::server::server;

pub use self::disk::{Disk, DiskCache, VirtioClient};
pub use self::ex_node::ExNode;
pub use self::extent::Extent;
pub use self::filesystem::FileSystem;
pub use self::header::Header;
pub use self::node::Node;

pub mod client;
mod ex_node;
mod extent;
mod filesystem;
mod header;
mod node;
mod disk;
mod mount;

pub const BLOCK_SIZE: u64 = 4096;
pub const SIGNATURE: &'static [u8; 8] = b"RedoxFS\0";
pub const VERSION: u64 = 4;
