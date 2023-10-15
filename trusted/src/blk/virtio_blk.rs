use alloc::boxed::Box;
use core::mem::size_of;

use tock_registers::interfaces::{Readable, Writeable};

use crate::common::mm::virt_to_phys;
use hardware::virtio_mmio::*;
use rpsyscall::get_tid;

struct VirtioBlkInnerMut {
  ring: VirtioRing,
  last_used: u16,
  queue: [Option<DiskRequest>; DRIVER_QUEUE_SIZE], // head desc -> disk_request
  desc_free: [bool; DRIVER_QUEUE_SIZE],
}

struct VirtioBlk {
  mmio: VirtioMmio,
  irq: usize,
  size_in_sector: usize,
  mutable: /* Mutex< */VirtioBlkInnerMut/* > */,
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

const DRIVER_QUEUE_SIZE: usize = 16;
const VRING_DESC_F_NEXT: u16 = 1;
const VRING_DESC_F_WRITE: u16 = 2;

impl VirtioBlk {
  fn new(base_addr: usize, irq_num: usize) -> Self {
    VirtioBlk {
      mmio: VirtioMmio::new(base_addr),
      irq: irq_num,
      size_in_sector: 0,
      mutable: /* Mutex::new( */VirtioBlkInnerMut {ring: VirtioRing {
        desc: [VirtioRingDesc {
          addr: 0,
          len: 0,
          flags: 0,
          next: 0,
        }; DRIVER_QUEUE_SIZE],
        driver: VirtioRingDriver {
          flags: 0,
          idx: 0,
          ring: [0xffff; DRIVER_QUEUE_SIZE],
        },
        device: VirtioRingDevice {
          flags: 0,
          idx: 0,
          ring: [VirtioRingDeviceElement { id: 0, len: 0 }; DRIVER_QUEUE_SIZE],
        },
      },
        last_used: 0,
        queue: [NONE_DISK_REQUEST; DRIVER_QUEUE_SIZE],
        desc_free: [true; DRIVER_QUEUE_SIZE],
      }/* ) */
    }
  }

  fn init(&mut self) {
    const MMIO_MAGIC: u32 = 0x74726976;
    let mmio = &self.mmio;
    if mmio.MagicValue.get() != MMIO_MAGIC || mmio.Version.get() != 2 || mmio.DeviceID.get() != 2 {
      panic!("Unsupported magic/version/device id");
    }
    mmio.Status.set(0);
    let mut status = mmio.Status.extract();
    status.modify(VirtioConfigStatus::ACKNOWLEDGE::SET);
    mmio.Status.set(status.get());
    status.modify(VirtioConfigStatus::DRIVER::SET);
    mmio.Status.set(status.get());

    mmio.DeviceFeaturesSel.set(0);
    info!("device feature low  {:08x}", mmio.DeviceFeatures.get());
    mmio.DeviceFeaturesSel.set(1);
    info!("device feature high {:08x}", mmio.DeviceFeatures.get());

    mmio.DriverFeaturesSel.set(0);
    mmio.DriverFeatures.write(
      VirtioBlkFeature::SEG_MAX::SET
        + VirtioBlkFeature::GEOMETRY::SET
        + VirtioBlkFeature::BLK_SIZE::SET
        + VirtioBlkFeature::TOPOLOGY::SET,
    );
    mmio.DriverFeaturesSel.set(1);
    mmio
      .DriverFeatures
      .write(VirtioBlkFeature::HIGH_VERSION_1::SET);

    status.modify(VirtioConfigStatus::DRIVER_OK::SET);
    mmio.Status.set(status.get());

    status.modify(VirtioConfigStatus::FEATURES_OK::SET);
    mmio.Status.set(status.get());

    self.setup_queue(0);
    trace!(
      "probe disk size lo{} / hi{} sectors",
      mmio.CapacityLow.get(),
      mmio.CapacityHigh.get()
    );
    let size = ((mmio.CapacityHigh.get() as usize) << 32) | mmio.CapacityLow.get() as usize;
    self.size_in_sector = size;
  }

  fn setup_queue(&self, idx: u32) {
    let mmio = &self.mmio;
    mmio.QueueSel.set(idx);
    let num = mmio.QueueNumMax.get();
    if num == 0 {
      panic!("queue num max is zero");
    }
    if num < DRIVER_QUEUE_SIZE as u32 {
      panic!("queue size not supported");
    }
    mmio.QueueNum.set(DRIVER_QUEUE_SIZE as u32);

    let ring = &self.mutable.ring;

    mmio
      .QueueDescLow
      .set(virt_to_phys(ring.desc.base_addr_usize()) as u32);
    mmio
      .QueueDescHigh
      .set((virt_to_phys(ring.desc.base_addr_usize()) >> 32) as u32);
    mmio
      .QueueDriverLow
      .set(virt_to_phys(ring.driver.base_addr_usize()) as u32);
    mmio
      .QueueDriverHigh
      .set((virt_to_phys(ring.driver.base_addr_usize()) >> 32) as u32);
    mmio
      .QueueDeviceLow
      .set(virt_to_phys(ring.device.base_addr_usize()) as u32);
    mmio
      .QueueDeviceHigh
      .set((virt_to_phys(ring.device.base_addr_usize()) >> 32) as u32);

    mmio.QueueReady.set(1);
  }

  fn submit_io(
    &mut self,
    sector: usize,
    count: usize,
    buf: usize,
    op: Operation,
    tid: usize,
  ) -> Result<(), &'static str> {
    if sector >= self.size_in_sector || sector + count >= self.size_in_sector {
      return Err("invalid sector");
    }
    let hdr = Box::new(VirtioBlkOutHdr {
      t: match op {
        Operation::Read => 0,
        Operation::Write => 1,
      },
      priority: 0,
      sector: sector as u64,
    });
    let status = Box::new(255u8);
    let mutable = &mut self.mutable;
    if let Some((ia, ib, ic)) = mutable.alloc_desc() {

      mutable.ring.desc[ia].addr = virt_to_phys(hdr.as_ref() as *const _ as usize) as u64;
      mutable.ring.desc[ia].len = size_of::<VirtioBlkOutHdr>() as u32;
      mutable.ring.desc[ia].flags = VRING_DESC_F_NEXT;
      mutable.ring.desc[ia].next = ib as u16;

      mutable.ring.desc[ib].addr = virt_to_phys(buf) as u64;
      mutable.ring.desc[ib].len = (512 * count) as u32;
      mutable.ring.desc[ib].flags = match op {
        Operation::Read => VRING_DESC_F_WRITE,
        Operation::Write => 0,
      } | VRING_DESC_F_NEXT;
      mutable.ring.desc[ib].next = ic as u16;

      mutable.ring.desc[ic].addr = virt_to_phys(status.as_ref() as *const u8 as usize) as u64;
      mutable.ring.desc[ic].len = 1;
      mutable.ring.desc[ic].flags = VRING_DESC_F_WRITE;
      mutable.ring.desc[ic].next = 0;

      let avail_idx = mutable.ring.driver.idx;
      mutable.ring.driver.ring[(avail_idx as usize) % DRIVER_QUEUE_SIZE] = ia as u16;

      mutable.queue[ia] = Some(DiskRequest {
        sector,
        count,
        buf,
        imp: hdr,
        status,
        src: tid,
      });
      let last_avail_idx = mutable.ring.driver.idx;
      mutable.ring.driver.idx = last_avail_idx.wrapping_add(1);
      
      let mmio = &self.mmio;
      mmio.QueueNotify.set(0); // queue num #0
      
    } else {
      return Err("queue full. no desc available");
    }

    Ok(())
  }

  fn complete_irq(&mut self) {
    const VIRTIO_BLK_S_OK: u8 = 0;
    const VIRTIO_BLK_S_IOERR: u8 = 1;
    const VIRTIO_BLK_S_UNSUPP: u8 = 2;
    let status = self.mmio.InterruptStatus.get();
    if status & 0b01 != 0 {
      // Used Buffer Notification: the device has used a buffer in at least one of the active virtual queues.
      loop {
        let mutable = &mut self.mutable;
        if mutable.last_used == mutable.ring.device.idx {
          break;
        }
        let comp_head = mutable.ring.device.ring[(mutable.last_used as usize) % DRIVER_QUEUE_SIZE].id as usize;
        if let Some(req) = mutable.queue[comp_head].take() {
          match *req.status {
            VIRTIO_BLK_S_OK => {
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
          let mut msg = rpsyscall::message::Message::default();
          msg.a = *req.status as usize;
          let _ = msg.send_to(req.src);
          mutable.free_desc(comp_head);
        } else {
          error!("head desc doesn't have corresponding DiskRequest");
        }
        mutable.last_used = mutable.last_used.wrapping_add(1);
      }
    }
    if status & 0b10 != 0 {
      // Configuration Change Notification
      error!("irq Configuration Change Notification not handled!");
    }
    self.mmio.InterruptACK.set(status);
  }

  // fn irq_thread(&mut self) {
  //   loop {
  //     self.poll_irq();
  //   }
  // }

  fn poll_irq(&mut self) {
    let _ = rpsyscall::event_wait(rpabi::event::EVENT_INTERRUPT, self.irq);
    self.complete_irq();
  }
}

impl VirtioBlkInnerMut {
  fn alloc_desc(&mut self) -> Option<(usize, usize, usize)> {
    for i in 0..DRIVER_QUEUE_SIZE {
      if self.desc_free[i] {
        for j in (i + 1)..DRIVER_QUEUE_SIZE {
          if self.desc_free[j] {
            for k in (j + 1)..DRIVER_QUEUE_SIZE {
              if self.desc_free[k] {
                self.desc_free[i] = false;
                self.desc_free[j] = false;
                self.desc_free[k] = false;
                return Some((i, j, k));
              }
            }
          }
        }
      }
    }
    None
  }

  fn free_desc(&mut self, head_idx: usize) {
    let a = &self.ring.desc[head_idx];
    let i = head_idx;
    if a.flags & VRING_DESC_F_NEXT != 0 {
      let j = a.next as usize;
      let b = &self.ring.desc[j];
      if b.flags & VRING_DESC_F_NEXT != 0 {
        let k = b.next as usize;
        let c = &self.ring.desc[k];
        assert_eq!(c.flags & VRING_DESC_F_NEXT, 0);
        self.desc_free[i] = true;
        self.desc_free[j] = true;
        self.desc_free[k] = true;
        return;
      }
    }
    panic!("inconsistent desc usage");
  }
}

#[repr(C)]
#[repr(align(4096))]
#[derive(Debug)]
struct VirtioRing {
  desc: [VirtioRingDesc; DRIVER_QUEUE_SIZE],
  driver: VirtioRingDriver,
  device: VirtioRingDevice,
}

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
  ring: [u16; DRIVER_QUEUE_SIZE],
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
  ring: [VirtioRingDeviceElement; DRIVER_QUEUE_SIZE],
}

#[repr(C)]
#[derive(Debug)]
pub struct VirtioBlkOutHdr {
  t: u32,
  priority: u32,
  sector: u64,
}

#[derive(Debug, Copy, Clone)]
pub enum Operation {
  Read,
  Write,
}

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
const NONE_DISK_REQUEST: Option<DiskRequest> = None;

// static VIRTIO_BLK: Once<VirtioBlk> = Once::new();

pub fn server() {
  info!("server started t{}", get_tid());
  rpsyscall::server_register(rpabi::server::SERVER_BLK).unwrap();
  // VIRTIO_BLK.call_once(|| {
  //   #[cfg(target_arch = "aarch64")]
  //   let mut virtio_blk = VirtioBlk::new(0x8_0000_0000 + 0x0a000000, 0x10 + 32);
  //   #[cfg(target_arch = "riscv64")]
  //   let virtio_blk = VirtioBlk::new(0x8_0000_0000 + 0x10001000, 0x1);
  //   virtio_blk.init();
  //   virtio_blk
  // });
  
  #[cfg(target_arch = "aarch64")]
  let mut virtio_blk = VirtioBlk::new(0x8_0000_0000 + 0x0a000000, 0x10 + 32);
  #[cfg(target_arch = "riscv64")]
  let mut virtio_blk = VirtioBlk::new(0x8_0000_0000 + 0x10001000, 0x1);
  virtio_blk.init();

  // let irq_thread = crate::common::thread::spawn(|| {
  //   VIRTIO_BLK.get().unwrap().irq_thread();
  // });
  loop {
    let (client_tid, msg) = rpsyscall::message::Message::receive().unwrap();
    // let virtio_blk = VIRTIO_BLK.get().unwrap();
    match msg.d {
      rpservapi::blk::action::READ | rpservapi::blk::action::WRITE => {
        let sector = msg.a;
        let count = msg.b;
        let buf = msg.c;
        let op = if msg.d == rpservapi::blk::action::READ {
          Operation::Read
        } else {
          Operation::Write
        };
        match virtio_blk.submit_io(sector, count, buf, op, client_tid) {
          Ok(_) => {
            // will reply in irq thread
            virtio_blk.poll_irq();
          },
          Err(_) => {
            let mut msg = rpsyscall::message::Message::default();
            msg.a = 0xff; // submit error
            let _ = msg.send_to(client_tid);
          }
        }
      }
      rpservapi::blk::action::SIZE => {
        let mut msg = rpsyscall::message::Message::default();
        msg.a = virtio_blk.size_in_sector * 512;
        let _ = msg.send_to(client_tid);
      }
      _ => {
        error!("unknown action {}", msg.d);
      }
    }
  }
  // let _ = irq_thread.join();
}
