use tock_registers::registers::*;
use tock_registers::*;

register_bitfields! {
    u32,
    pub VirtioConfigStatus [
        /// We have seen device and processed generic fields (VIRTIO_CONFIG_F_VIRTIO)
        ACKNOWLEDGE OFFSET(0) NUMBITS(1) [],
        /// We have found a driver for the device
        DRIVER      OFFSET(1) NUMBITS(1) [],
        /// Driver has used its parts of the config, and is happy
        DRIVER_OK   OFFSET(2) NUMBITS(1) [],
        /// Driver has finished configuring features
        FEATURES_OK OFFSET(3) NUMBITS(1) [],
        /// Device entered invalid state, driver must reset it
        NEEDS_RESET OFFSET(6) NUMBITS(1) [],
        /// We've given up on this device
        FAILED      OFFSET(7) NUMBITS(1) [],
    ],
    pub VirtioBlkFeature [
        /// v1.0 compliant
        HIGH_VERSION_1 OFFSET(0) NUMBITS(1) [],
        /// Indicates maximum segment size
        SIZE_MAX OFFSET(1) NUMBITS(1) [],
        /// Indicates maximum # of segments
        SEG_MAX  OFFSET(2) NUMBITS(1) [],
        /// Legacy geometry available
        GEOMETRY OFFSET(4) NUMBITS(1) [],
        /// Disk is read-only
        RO       OFFSET(5) NUMBITS(1) [],
        /// Block size of disk is available
        BLK_SIZE OFFSET(6) NUMBITS(1) [],
        /// Topology information is available
        TOPOLOGY OFFSET(10) NUMBITS(1) [],
        /// Support more than one vq
        MQ       OFFSET(12) NUMBITS(1) [],
    ]
}

register_structs! {
  #[allow(non_snake_case)]
  pub VirtioMmioBlock {
    (0x000 => pub MagicValue: ReadOnly<u32>),
    (0x004 => pub Version: ReadOnly<u32>),
    (0x008 => pub DeviceID: ReadOnly<u32>),
    (0x00c => pub VendorID: ReadOnly<u32>),
    (0x010 => pub DeviceFeatures: ReadOnly<u32, VirtioBlkFeature::Register>),
    (0x014 => pub DeviceFeaturesSel: WriteOnly<u32>),
    (0x018 => _reserved_0),
    (0x020 => pub DriverFeatures: WriteOnly<u32, VirtioBlkFeature::Register>),
    (0x024 => pub DriverFeaturesSel: WriteOnly<u32>),
    (0x028 => _reserved_1),
    (0x030 => pub QueueSel: WriteOnly<u32>),
    (0x034 => pub QueueNumMax: ReadOnly<u32>),
    (0x038 => pub QueueNum: WriteOnly<u32>),
    (0x03c => _reserved_2),
    (0x044 => pub QueueReady: ReadWrite<u32>),
    (0x048 => _reserved_3),
    (0x050 => pub QueueNotify: WriteOnly<u32>),
    (0x054 => _reserved_4),
    (0x060 => pub InterruptStatus: ReadOnly<u32>),
    (0x064 => pub InterruptACK: WriteOnly<u32>),
    (0x068 => _reserved_5),
    (0x070 => pub Status: ReadWrite<u32, VirtioConfigStatus::Register>),
    (0x074 => _reserved_6),
    (0x080 => pub QueueDescLow: WriteOnly<u32>),
    (0x084 => pub QueueDescHigh: WriteOnly<u32>),
    (0x088 => _reserved_7),
    (0x090 => pub QueueDriverLow: WriteOnly<u32>),
    (0x094 => pub QueueDriverHigh: WriteOnly<u32>),
    (0x098 => _reserved_8),
    (0x0a0 => pub QueueDeviceLow: WriteOnly<u32>),
    (0x0a4 => pub QueueDeviceHigh: WriteOnly<u32>),
    (0x0a8 => _reserved_9),
    (0x0fc => pub ConfigGeneration: ReadOnly<u32>),
    (0x100 => pub CapacityLow: ReadOnly<u32>),
    (0x104 => pub CapacityHigh: ReadOnly<u32>),
    (0x108 => pub SizeMax: ReadOnly<u32>),
    (0x10c => pub SegMax: ReadOnly<u32>),
    (0x110 => _reserved_config),
    (0x200 => @END),
  }
}

pub struct VirtioMmio {
  base_addr: usize,
}

impl core::ops::Deref for VirtioMmio {
  type Target = VirtioMmioBlock;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.ptr() }
  }
}

impl VirtioMmio {
  pub const fn new(base_addr: usize) -> Self {
    VirtioMmio { base_addr }
  }

  fn ptr(&self) -> *const VirtioMmioBlock {
    self.base_addr as *const _
  }
}
