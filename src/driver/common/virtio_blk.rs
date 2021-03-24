use crate::arch::{Address, PAGE_SIZE, PAGE_SHIFT};
use core::mem::size_of;
use alloc::boxed::Box;

const VIRTIO_MMIO_BASE: usize = 0x0a003000;

#[repr(C)]
struct VirtioMMio {
    magic_value: u32,
    mmio_version: u32,
    device_id: u32,
    vendor_id: u32,
    device_features: u32,
    host_features_sel: u32,
    _pad0: u32,
    _pad1: u32,
    driver_features: u32,
    guest_features_sel: u32,
    guest_page_size: u32,
    // legacy only
    _pad2: u32,
    queue_sel: u32,
    queue_num_max: u32,
    queue_num: u32,
    queue_align: u32,
    // legacy only
    queue_pfn: u32,
    // legacy only
    queue_ready: u32,
    _pad3: u32,
    _pad4: u32,
    queue_notify: u32,
    _pad5: [u32; 3],
    interrupt_status: u32,
    interrupt_ack: u32,
    _pad6: [u32; 2],
    status: u32,
    _pad7: [u32; 3],
    queue_desc_low: u32,
    queue_desc_high: u32,
    _pad8: [u32; 2],
    queue_avail_low: u32,
    queue_avail_high: u32,
    _pad9: [u32; 2],
    queue_used_low: u32,
    queue_used_high: u32,
    // ignore left
}

/* We have seen device and processed generic fields (VIRTIO_CONFIG_F_VIRTIO) */
const VIRTIO_CONFIG_S_ACKNOWLEDGE: u32 = 1;
/* We have found a driver for the device */
const VIRTIO_CONFIG_S_DRIVER: u32 = 2;
/* Driver has used its parts of the config, and is happy */
const VIRTIO_CONFIG_S_DRIVER_OK: u32 = 4;
/* Driver has finished configuring features */
const VIRTIO_CONFIG_S_FEATURES_OK: u32 = 8;
/* Device entered invalid state, driver must reset it */
const VIRTIO_CONFIG_S_NEEDS_RESET: u32 = 0x40;
/* We've given up on this device */
const VIRTIO_CONFIG_S_FAILED: u32 = 0x80;

/* v1.0 compliant */
const VIRTIO_F_VERSION_1: u32 = 32;

/* Feature bits */
const VIRTIO_BLK_F_SIZE_MAX: u32 = 1;  /* Indicates maximum segment size */
const VIRTIO_BLK_F_SEG_MAX: u32 = 2;   /* Indicates maximum # of segments */
const VIRTIO_BLK_F_GEOMETRY: u32 = 4;  /* Legacy geometry available */
const VIRTIO_BLK_F_RO: u32 = 5;        /* Disk is read-only */
const VIRTIO_BLK_F_BLK_SIZE: u32 = 6;  /* Block size of disk is available */
const VIRTIO_BLK_F_TOPOLOGY: u32 = 10; /* Topology information is available */
const VIRTIO_BLK_F_MQ: u32 = 12;       /* Support more than one vq */

/* Legacy feature bits */
const VIRTIO_BLK_F_BARRIER: u32 = 0;    /* Does host support barriers? */
const VIRTIO_BLK_F_SCSI: u32 = 7;        /* Supports scsi command passthru */
const VIRTIO_BLK_F_FLUSH: u32 = 9;      /* Flush command supported */
const VIRTIO_BLK_F_CONFIG_WCE: u32 = 11; /* Writeback mode available in config */

/* Can the device handle any descriptor layout? */
const VIRTIO_F_ANY_LAYOUT: u32 = 27;
/*
 * The Guest publishes the used index for which it expects an interrupt
 * at the end of the avail ring. Host should ignore the avail->flags field.
 *
 * The Host publishes the avail index for which it expects a kick
 * at the end of the used ring. Guest should ignore the used->flags field.
 */
const VIRTIO_RING_F_EVENT_IDX: u32 = 29;
/* We support indirect buffer descriptors */
const VIRTIO_RING_F_INDIRECT_DESC: u32 = 28;


fn virtio_mmio() -> &'static mut VirtioMMio {
    unsafe { ((VIRTIO_MMIO_BASE + 7 * 0x200).pa2kva() as *mut VirtioMMio).as_mut() }.unwrap()
}

trait BaseAddr {
    fn base_addr_u64(&self) -> u64;
    fn base_addr_usize(&self) -> usize;
}

impl<T> BaseAddr for T {
    fn base_addr_u64(&self) -> u64 {
        self as *const T as u64
    }
    fn base_addr_usize(&self) -> usize {
        self as *const T as usize
    }
}

fn setup_queue(idx: u32) {
    let mmio = virtio_mmio();
    mmio.queue_sel = idx;
    if mmio.queue_pfn != 0 {
        panic!("queue already set up");
    }
    let num = mmio.queue_num_max;
    if num == 0 {
        panic!("queue num max is zero");
    }
    if num < QUEUE_SIZE as u32 {
        panic!("queue size not supported");
    }
    mmio.queue_num = QUEUE_SIZE as u32;
    unsafe {
        mmio.queue_desc_low = VIRTIO_RING.desc.base_addr_usize().kva2pa() as u32;
        mmio.queue_desc_high = (VIRTIO_RING.desc.base_addr_usize().kva2pa() >> 32) as u32;
        mmio.queue_avail_low = VIRTIO_RING.avail.base_addr_usize().kva2pa() as u32;
        mmio.queue_avail_high = (VIRTIO_RING.avail.base_addr_usize().kva2pa() >> 32) as u32;
        mmio.queue_used_low = VIRTIO_RING.used.base_addr_usize().kva2pa() as u32;
        mmio.queue_used_high = (VIRTIO_RING.used.base_addr_usize().kva2pa() >> 32) as u32;
    }
    mmio.queue_ready = 1;
}

pub fn init() {
    let mmio = virtio_mmio();
    if mmio.magic_value != 0x74726976
        || mmio.mmio_version != 2
        || mmio.device_id != 2
        || mmio.vendor_id != 0x554d4551 {
        panic!("could not find virtio disk");
    }
    let mut status: u32 = 0;
    status |= VIRTIO_CONFIG_S_ACKNOWLEDGE;
    mmio.status = status;
    status |= VIRTIO_CONFIG_S_DRIVER;
    mmio.status = status;

    mmio.host_features_sel = 0;
    let mut features: u64 = mmio.device_features as u64;
    mmio.host_features_sel = 1;
    features |= (mmio.device_features as u64) << 32;

    // TODO: support more features
    features = 1 << VIRTIO_F_VERSION_1;

    mmio.guest_features_sel = 0;
    mmio.driver_features = features as u32;
    mmio.guest_features_sel = 1;
    mmio.driver_features = (features >> 32) as u32;

    status |= VIRTIO_CONFIG_S_FEATURES_OK;
    mmio.status = status;

    status |= VIRTIO_CONFIG_S_DRIVER_OK;
    mmio.status = status;

    setup_queue(0);
}

const QUEUE_SIZE: usize = 16;

#[repr(C)]
#[repr(align(4096))]
#[derive(Debug)]
struct VirtioRing {
    desc: [VirtioRingDesc; QUEUE_SIZE],
    avail: VirtioRingAvail,
    used: VirtioRingUsed,
}

static mut VIRTIO_RING: VirtioRing = VirtioRing {
    desc: [VirtioRingDesc {
        addr: 0,
        len: 0,
        flags: 0,
        next: 0,
    }; QUEUE_SIZE],
    avail: VirtioRingAvail {
        flags: 0,
        idx: 0,
        ring: [0; QUEUE_SIZE],
    },
    used: VirtioRingUsed {
        flags: 0,
        idx: 0,
        ring: [VirtioRingUsedElement {
            id: 0,
            len: 0,
        }; QUEUE_SIZE],
    },
};

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct VirtioRingDesc {
    addr: u64,
    len: u32,
    flags: u16,
    next: u16,
}

#[repr(C)]
#[derive(Debug)]
struct VirtioRingAvail {
    flags: u16,
    idx: u16,
    ring: [u16; QUEUE_SIZE],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct VirtioRingUsedElement {
    id: u32,
    len: u32,
}

#[repr(C)]
#[repr(align(4096))]
#[derive(Debug)]
struct VirtioRingUsed {
    flags: u16,
    idx: u16,
    ring: [VirtioRingUsedElement; QUEUE_SIZE],
}

pub enum Operation {
    Read,
    Write,
}

#[repr(C)]
pub struct VirtioBlkOutHdr {
    t: u32,
    priority: u32,
    sector: u64,
}

/* This marks a buffer as continuing via the next field */
const VRING_DESC_F_NEXT: u16 = 1;
/* This marks a buffer as write-only (otherwise read-only) */
const VRING_DESC_F_WRITE: u16 = 2;
/* This means the buffer contains a list of buffer descriptors */
const VRING_DESC_F_INDIRECT: u16 = 4;

pub fn io(sector: usize, count: usize, buf: *const u8, op: Operation) -> (Box<VirtioBlkOutHdr>, Box<u8>) {
    let hdr = Box::new(VirtioBlkOutHdr {
        t: match op {
            Operation::Read => 0,
            Operation::Write => 1,
        },
        priority: 0,
        sector: sector as u64,
    });
    let status = Box::new(255u8);

    let desc = unsafe { VIRTIO_RING.desc.get_unchecked_mut(0) };
    desc.addr = (hdr.as_ref() as *const VirtioBlkOutHdr as usize).kva2pa() as u64;
    desc.len = unsafe { size_of::<VirtioBlkOutHdr>() } as u32;
    desc.flags = VRING_DESC_F_NEXT;
    desc.next = 1;
    let desc = unsafe { VIRTIO_RING.desc.get_unchecked_mut(1) };
    desc.addr = (buf as usize).kva2pa() as u64;
    desc.len = (512 * count) as u32;
    desc.flags = match op {
        Operation::Read => VRING_DESC_F_WRITE,
        Operation::Write => 0
    };
    desc.flags |= VRING_DESC_F_NEXT;
    desc.next = 2;

    let desc = unsafe { VIRTIO_RING.desc.get_unchecked_mut(2) };
    desc.addr = (status.as_ref() as *const u8 as usize).kva2pa() as u64;
    desc.len = 1;
    desc.flags = VRING_DESC_F_WRITE;
    desc.next = 0;

    let avail = unsafe { &mut VIRTIO_RING.avail };
    avail.ring[(avail.idx as usize) % QUEUE_SIZE] = 0;
    // barrier
    avail.idx += 1;

    let mmio = virtio_mmio();

    mmio.queue_notify = 0; // queue num

    loop {
        if mmio.interrupt_status == 1 {
            break;
        }
    }

    (hdr, status)
}
