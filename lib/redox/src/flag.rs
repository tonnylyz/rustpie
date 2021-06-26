use bitflags::bitflags as inner_bitflags;
// from redox_syscall flag.rs
macro_rules! bitflags {
    (
        $(#[$outer:meta])*
        pub struct $BitFlags:ident: $T:ty {
            $(
                $(#[$inner:ident $($args:tt)*])*
                const $Flag:ident = $value:expr;
            )+
        }
    ) => {
        // First, use the inner bitflags
        inner_bitflags! {
            #[derive(Default)]
            $(#[$outer])*
            pub struct $BitFlags: $T {
                $(
                    $(#[$inner $($args)*])*
                    const $Flag = $value;
                )+
            }
        }

        // Secondly, re-export all inner constants
        // (`pub use self::Struct::*` doesn't work)
        $(
            $(#[$inner $($args)*])*
            pub const $Flag: $BitFlags = $BitFlags::$Flag;
        )+
    }
}

bitflags! {
    pub struct CloneFlags: usize {
        const CLONE_VM = 0x100;
        const CLONE_FS = 0x200;
        const CLONE_FILES = 0x400;
        const CLONE_SIGHAND = 0x800;
        const CLONE_VFORK = 0x4000;
        const CLONE_THREAD = 0x10000;
        const CLONE_STACK = 0x1000_0000;
    }
}

pub const CLOCK_REALTIME: usize = 1;
pub const CLOCK_MONOTONIC: usize = 4;

bitflags! {
    pub struct EventFlags: usize {
        const EVENT_NONE = 0;
        const EVENT_READ = 1;
        const EVENT_WRITE = 2;
    }
}

pub const F_DUPFD: usize = 0;
pub const F_GETFD: usize = 1;
pub const F_SETFD: usize = 2;
pub const F_GETFL: usize = 3;
pub const F_SETFL: usize = 4;

pub const FUTEX_WAIT: usize = 0;
pub const FUTEX_WAKE: usize = 1;
pub const FUTEX_REQUEUE: usize = 2;

bitflags! {
    pub struct MapFlags: usize {
        const PROT_NONE = 0x0000_0000;
        const PROT_EXEC = 0x0001_0000;
        const PROT_WRITE = 0x0002_0000;
        const PROT_READ = 0x0004_0000;

        const MAP_SHARED = 0x0001;
        const MAP_PRIVATE = 0x0002;

        /// Only accepted for mmap2(2).
        const MAP_FIXED = 0x0004;
        const MAP_FIXED_NOREPLACE = 0x000C;
    }
}

pub const MODE_TYPE: u16 = 0xF000;
pub const MODE_DIR: u16 = 0x4000;
pub const MODE_FILE: u16 = 0x8000;
pub const MODE_SYMLINK: u16 = 0xA000;
pub const MODE_FIFO: u16 = 0x1000;
pub const MODE_CHR: u16 = 0x2000;

pub const MODE_PERM: u16 = 0x0FFF;
pub const MODE_SETUID: u16 = 0o4000;
pub const MODE_SETGID: u16 = 0o2000;

pub const O_RDONLY: usize =     0x0001_0000;
pub const O_WRONLY: usize =     0x0002_0000;
pub const O_RDWR: usize =       0x0003_0000;
pub const O_NONBLOCK: usize =   0x0004_0000;
pub const O_APPEND: usize =     0x0008_0000;
pub const O_SHLOCK: usize =     0x0010_0000;
pub const O_EXLOCK: usize =     0x0020_0000;
pub const O_ASYNC: usize =      0x0040_0000;
pub const O_FSYNC: usize =      0x0080_0000;
pub const O_CLOEXEC: usize =    0x0100_0000;
pub const O_CREAT: usize =      0x0200_0000;
pub const O_TRUNC: usize =      0x0400_0000;
pub const O_EXCL: usize =       0x0800_0000;
pub const O_DIRECTORY: usize =  0x1000_0000;
pub const O_STAT: usize =       0x2000_0000;
pub const O_SYMLINK: usize =    0x4000_0000;
pub const O_NOFOLLOW: usize =   0x8000_0000;
pub const O_ACCMODE: usize =    O_RDONLY | O_WRONLY | O_RDWR;


pub const SEEK_SET: usize = 0;
pub const SEEK_CUR: usize = 1;
pub const SEEK_END: usize = 2;
