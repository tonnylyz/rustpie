use alloc::boxed::Box;
use alloc::vec::Vec;
use core::mem::size_of;
use tock_registers::*;
use tock_registers::registers::*;
use spin::{Mutex, Once};
use libtrusted::mm::virt_to_phys;
use microcall::get_tid;

use tock_registers::interfaces::{Writeable, Readable};

#[cfg(target_arch = "aarch64")]
const VIRTIO_MMIO_BASE: usize = 0x8_0000_0000 + 0x0a000000;

#[cfg(target_arch = "riscv64")]
const VIRTIO_MMIO_BASE: usize = 0x8_0000_0000 + 0x10001000;

register_structs! {
  #[allow(non_snake_case)]
  VirtioMmioBlock {
    (0x000 => MagicValue: ReadOnly<u32>),
    (0x004 => Version: ReadOnly<u32>),
    (0x008 => DeviceID: ReadOnly<u32>),
    (0x00c => VendorID: ReadOnly<u32>),
    (0x010 => DeviceFeatures: ReadOnly<u32>),
    (0x014 => DeviceFeaturesSel: WriteOnly<u32>),
    (0x018 => _reserved_0),
    (0x020 => DriverFeatures: WriteOnly<u32>),
    (0x024 => DriverFeaturesSel: WriteOnly<u32>),
    (0x028 => _reserved_1),
    (0x030 => QueueSel: WriteOnly<u32>),
    (0x034 => QueueNumMax: ReadOnly<u32>),
    (0x038 => QueueNum: WriteOnly<u32>),
    (0x03c => _reserved_2),
    (0x044 => QueueReady: ReadWrite<u32>),
    (0x048 => _reserved_3),
    (0x050 => QueueNotify: WriteOnly<u32>),
    (0x054 => _reserved_4),
    (0x060 => InterruptStatus: ReadOnly<u32>),
    (0x064 => InterruptACK: WriteOnly<u32>),
    (0x068 => _reserved_5),
    (0x070 => Status: ReadWrite<u32>),
    (0x074 => _reserved_6),
    (0x080 => QueueDescLow: WriteOnly<u32>),
    (0x084 => QueueDescHigh: WriteOnly<u32>),
    (0x088 => _reserved_7),
    (0x090 => QueueDriverLow: WriteOnly<u32>),
    (0x094 => QueueDriverHigh: WriteOnly<u32>),
    (0x098 => _reserved_8),
    (0x0a0 => QueueDeviceLow: WriteOnly<u32>),
    (0x0a4 => QueueDeviceHigh: WriteOnly<u32>),
    (0x0a8 => _reserved_9),
    (0x0fc => ConfigGeneration: ReadOnly<u32>),
    (0x100 => CapacityLow: ReadOnly<u32>),
    (0x104 => CapacityHigh: ReadOnly<u32>),
    (0x108 => SizeMax: ReadOnly<u32>),
    (0x10c => SegMax: ReadOnly<u32>),
    (0x110 => _reserved_config),
    (0x200 => @END),
  }
}

struct VirtioMmio {
  base_addr: usize,
  disk_size: Once<usize>,
}

impl core::ops::Deref for VirtioMmio {
  type Target = VirtioMmioBlock;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.ptr() }
  }
}

impl VirtioMmio {
  const fn new(base_addr: usize) -> Self {
    VirtioMmio { base_addr, disk_size: Once::new() }
  }

  fn ptr(&self) -> *const VirtioMmioBlock {
    self.base_addr as *const _
  }
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
// const VIRTIO_CONFIG_S_NEEDS_RESET: u32 = 0x40;
/* We've given up on this device */
// const VIRTIO_CONFIG_S_FAILED: u32 = 0x80;

/* v1.0 compliant */
const VIRTIO_F_VERSION_1: usize = 32;

/* Feature bits */
// const VIRTIO_BLK_F_SIZE_MAX: usize = 1;  /* Indicates maximum segment size */
const VIRTIO_BLK_F_SEG_MAX: usize = 2;   /* Indicates maximum # of segments */
const VIRTIO_BLK_F_GEOMETRY: usize = 4;  /* Legacy geometry available */
// const VIRTIO_BLK_F_RO: usize = 5;        /* Disk is read-only */
const VIRTIO_BLK_F_BLK_SIZE: usize = 6;  /* Block size of disk is available */
const VIRTIO_BLK_F_TOPOLOGY: usize = 10; /* Topology information is available */
// const VIRTIO_BLK_F_MQ: u32 = 12;       /* Support more than one vq */

/* Legacy feature bits */
// const VIRTIO_BLK_F_BARRIER: usize = 0;     /* Does host support barriers? */
// const VIRTIO_BLK_F_SCSI: usize = 7;        /* Supports scsi command passthru */
// const VIRTIO_BLK_F_FLUSH: usize = 9;          /* Flush command supported */
// const VIRTIO_BLK_F_CONFIG_WCE: usize = 11;    /* Writeback mode available in config */

// const VIRTIO_BLK_F_DISCARD: usize = 13;
// const VIRTIO_BLK_F_WRITE_ZEROES: usize = 14;
/* Can the device handle any descriptor layout? */
// const VIRTIO_F_ANY_LAYOUT: u32 = 27;
/*
 * The Guest publishes the used index for which it expects an interrupt
 * at the end of the avail ring. Host should ignore the avail->flags field.
 *
 * The Host publishes the avail index for which it expects a kick
 * at the end of the used ring. Guest should ignore the used->flags field.
 */
// const VIRTIO_RING_F_EVENT_IDX: usize = 29;
/* We support indirect buffer descriptors */
// const VIRTIO_RING_F_INDIRECT_DESC: usize = 28;

static VIRTIO_MMIO: VirtioMmio = VirtioMmio::new(VIRTIO_MMIO_BASE);


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
  let mmio = &VIRTIO_MMIO;
  mmio.QueueSel.set(idx);
  let num = mmio.QueueNumMax.get();
  if num == 0 {
    panic!("queue num max is zero");
  }
  if num < QUEUE_SIZE as u32 {
    panic!("queue size not supported");
  }
  mmio.QueueNum.set(QUEUE_SIZE as u32);

  let ring = VIRTIO_RING.lock();

  mmio.QueueDescLow.set(virt_to_phys(ring.desc.base_addr_usize()) as u32);
  mmio.QueueDescHigh.set((virt_to_phys(ring.desc.base_addr_usize()) >> 32) as u32);
  mmio.QueueDriverLow.set(virt_to_phys(ring.driver.base_addr_usize()) as u32);
  mmio.QueueDriverHigh.set((virt_to_phys(ring.driver.base_addr_usize()) >> 32) as u32);
  mmio.QueueDeviceLow.set(virt_to_phys(ring.device.base_addr_usize()) as u32);
  mmio.QueueDeviceHigh.set((virt_to_phys(ring.device.base_addr_usize()) >> 32) as u32);

  mmio.QueueReady.set(1);
}

pub fn init() {
  let mmio = &VIRTIO_MMIO;
  if mmio.MagicValue.get() != 0x74726976
    || mmio.Version.get() != 2
    || mmio.DeviceID.get() != 2 {
    panic!("could not find virtio disk");
  }
  let mut status: u32 = 0;
  mmio.Status.set(status);
  status |= VIRTIO_CONFIG_S_ACKNOWLEDGE;
  mmio.Status.set(status);
  status |= VIRTIO_CONFIG_S_DRIVER;
  mmio.Status.set(status);

  mmio.DeviceFeaturesSel.set(0);
  // info!("device feature low  {:08x}", mmio.DeviceFeatures.get());
  mmio.DeviceFeaturesSel.set(1);
  // info!("device feature high {:08x}", mmio.DeviceFeatures.get());

  let features: u64 =
    1 << VIRTIO_F_VERSION_1
    | 1 << VIRTIO_BLK_F_SEG_MAX
    | 1 << VIRTIO_BLK_F_GEOMETRY
    | 1 << VIRTIO_BLK_F_BLK_SIZE
    | 1 << VIRTIO_BLK_F_TOPOLOGY;


  mmio.DriverFeaturesSel.set(0);
  mmio.DriverFeatures.set(features as u32);
  mmio.DriverFeaturesSel.set(1);
  mmio.DriverFeatures.set((features >> 32) as u32);

  status |= VIRTIO_CONFIG_S_DRIVER_OK;
  mmio.Status.set(status);

  status |= VIRTIO_CONFIG_S_FEATURES_OK;
  mmio.Status.set(status);

  setup_queue(0);
  // info!("probe disk size {} / {} sectors", mmio.CapacityLow.get(), mmio.CapacityHigh.get());
  let size = ((mmio.CapacityHigh.get() as usize) << 32) | mmio.CapacityLow.get() as usize;
  mmio.disk_size.call_once(|| size);
}

const QUEUE_SIZE: usize = 16;

#[repr(C)]
#[repr(align(4096))]
#[derive(Debug)]
struct VirtioRing {
  desc: [VirtioRingDesc; QUEUE_SIZE],
  driver: VirtioRingDriver,
  device: VirtioRingDevice,
}

static VIRTIO_RING: Mutex<VirtioRing> = Mutex::new(VirtioRing {
  desc: [VirtioRingDesc {
    addr: 0,
    len: 0,
    flags: 0,
    next: 0,
  }; QUEUE_SIZE],
  driver: VirtioRingDriver {
    flags: 0,
    idx: 0,
    ring: [0; QUEUE_SIZE],
  },
  device: VirtioRingDevice {
    flags: 0,
    idx: 0,
    ring: [VirtioRingDeviceElement {
      id: 0,
      len: 0,
    }; QUEUE_SIZE],
  },
});

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
struct VirtioRingDriver {
  flags: u16,
  idx: u16,
  ring: [u16; QUEUE_SIZE],
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
struct VirtioRingDeviceElement {
  id: u32,
  len: u32,
}

#[repr(C)]
#[repr(align(4096))]
#[derive(Debug)]
struct VirtioRingDevice {
  flags: u16,
  idx: u16,
  ring: [VirtioRingDeviceElement; QUEUE_SIZE],
}

pub enum Operation {
  Read,
  Write,
}

#[repr(C)]
#[derive(Debug)]
pub struct VirtioBlkOutHdr {
  t: u32,
  priority: u32,
  sector: u64,
}

/* This marks a buffer as continuing via the next field */
const VRING_DESC_F_NEXT: u16 = 1;
/* This marks a buffer as write-only (otherwise read-only) */
const VRING_DESC_F_WRITE: u16 = 2;

#[derive(Debug)]
#[allow(dead_code)]
pub struct DiskRequest {
  sector: usize,
  count: usize,
  buf: usize,
  imp: Box<VirtioBlkOutHdr>,
  status: Box<u8>,
  src: usize,
}

pub struct Disk {
  last_used: u16,
  requests: Vec<DiskRequest>,
}

static DISK: Mutex<Disk> = Mutex::new(Disk {
  last_used: 0,
  requests: Vec::new(),
});

fn io(sector: usize, count: usize, buf: usize, op: Operation, src: usize) {
  let hdr = Box::new(VirtioBlkOutHdr {
    t: match op {
      Operation::Read => 0,
      Operation::Write => 1,
    },
    priority: 0,
    sector: sector as u64,
  });
  let status = Box::new(255u8);
  let mut ring = VIRTIO_RING.lock();

  let desc = ring.desc.get_mut(0).unwrap();
  desc.addr = virt_to_phys(hdr.as_ref() as *const VirtioBlkOutHdr as usize) as u64;
  desc.len = size_of::<VirtioBlkOutHdr>() as u32;
  desc.flags = VRING_DESC_F_NEXT;
  desc.next = 1;
  let desc = ring.desc.get_mut(1).unwrap();
  desc.addr = virt_to_phys(buf) as u64;
  desc.len = (512 * count) as u32;
  desc.flags = match op {
    Operation::Read => VRING_DESC_F_WRITE,
    Operation::Write => 0
  };
  desc.flags |= VRING_DESC_F_NEXT;
  desc.next = 2;

  let desc = ring.desc.get_mut(2).unwrap();
  desc.addr = virt_to_phys(status.as_ref() as *const u8 as usize) as u64;
  desc.len = 1;
  desc.flags = VRING_DESC_F_WRITE;
  desc.next = 0;

  let avail = &mut ring.driver;
  avail.ring[(avail.idx as usize) % QUEUE_SIZE] = 0;
  // barrier
  avail.idx = avail.idx.wrapping_add(1);

  let mut disk = DISK.lock();
  disk.requests.push(DiskRequest {
    sector,
    count,
    buf,
    imp: hdr,
    status,
    src
  });

  let mmio = &VIRTIO_MMIO;
  mmio.QueueNotify.set(0); // queue num #0
}

const VIRTIO_BLK_S_OK: u8 = 0;
const VIRTIO_BLK_S_IOERR: u8 = 1;
const VIRTIO_BLK_S_UNSUPP: u8 = 2;

fn irq() {
  let mmio = &VIRTIO_MMIO;
  let status = mmio.InterruptStatus.get();
  if status & 0b01 != 0 {
    // Used Buffer Notification: the device has used a buffer in at least one of the active virtual queues.

    let ring = VIRTIO_RING.lock();
    let mut disk = DISK.lock();
    let used = &ring.device;

    loop {
      if disk.last_used == used.idx {
        break;
      }
      if disk.requests.is_empty() {
        error!("irq no requests record");
        break;
      }
      if used.ring[(disk.last_used as usize) % QUEUE_SIZE].id != 0 {
        error!("irq unexpected ring desc head, only #0 is used");
        break;
      }
      let req = &disk.requests[0];
      match *req.status {
        VIRTIO_BLK_S_OK => {
          {
            let msg = microcall::message::Message::default();
            let _ = msg.send_to(req.src);
          }
        }
        VIRTIO_BLK_S_IOERR => {
          error!("irq status io err {:#x?}", req);
        }
        VIRTIO_BLK_S_UNSUPP => {
          error!("irq status unsupported {:#x?}", req);
        }
        x => {
          error!("irq unknown status {}", x);
        }
      }
      disk.requests.remove(0);
      disk.last_used += 1;
    }
  }
  if status & 0b10 != 0 {
    // Configuration Change Notification
    error!("irq Configuration Change Notification not handled!");
  }
  mmio.InterruptACK.set(status);
}

fn wait_for_irq() {
  #[cfg(target_arch = "aarch64")]
    let _ = microcall::event_wait(common::event::EVENT_INTERRUPT, 0x10 + 32);

  #[cfg(target_arch = "riscv64")]
    let _ = microcall::event_wait(common::event::EVENT_INTERRUPT, 0x01);
}

pub fn server() {
  init();
  info!("server started t{}",  get_tid());
  microcall::server_register(common::server::SERVER_BLK).unwrap();

  loop {
    let (client_tid, msg) = microcall::message::Message::receive().unwrap();
    trace!("recv {:x?}", (client_tid, msg));
    if msg.d == 0 || msg.d == 1 {
      let sector = msg.a;
      let count = msg.b;
      let buf = msg.c;
      let op = if msg.d == 0 { Operation::Read } else { Operation::Write };
      io(sector, count, buf, op, client_tid);
      wait_for_irq();
      irq();
    } else if msg.d == 2 {
      let mut msg = microcall::message::Message::default();
      msg.a = match VIRTIO_MMIO.disk_size.get() {
        None => 0,
        Some(s) => *s * 512,
      };
      let _ = msg.send_to(client_tid);
    } else {
      let mut msg = microcall::message::Message::default();
      msg.a = 1;
      let _ = msg.send_to(client_tid);
    }
  }
}
