//! SD card slot access (in SPI mode) on Maix Go
use core::convert::TryInto;
use libtrusted::mm::{valloc, virt_to_phys};

pub struct SDCard;

/*
 * Start Data tokens:
 *         Tokens (necessary because at nop/idle (and CS active) only 0xff is
 *         on the data/command line)
 */
/** Data token start byte, Start Single Block Read */
pub const SD_START_DATA_SINGLE_BLOCK_READ: u8 = 0xFE;
/** Data token start byte, Start Multiple Block Read */
pub const SD_START_DATA_MULTIPLE_BLOCK_READ: u8 = 0xFE;
/** Data token start byte, Start Single Block Write */
pub const SD_START_DATA_SINGLE_BLOCK_WRITE: u8 = 0xFE;
/** Data token start byte, Start Multiple Block Write */
pub const SD_START_DATA_MULTIPLE_BLOCK_WRITE: u8 = 0xFC;

pub const SEC_LEN: usize = 512;

/** SD commands */
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum CMD {
  /** Software reset */
  CMD0 = 0,
  /** Check voltage range (SDC V2) */
  CMD8 = 8,
  /** Read CSD register */
  CMD9 = 9,
  /** Read CID register */
  CMD10 = 10,
  /** Stop to read data */
  CMD12 = 12,
  /** Change R/W block size */
  CMD16 = 16,
  /** Read block */
  CMD17 = 17,
  /** Read multiple blocks */
  CMD18 = 18,
  /** Number of blocks to erase (SDC) */
  ACMD23 = 23,
  /** Write a block */
  CMD24 = 24,
  /** Write multiple blocks */
  CMD25 = 25,
  /** Initiate initialization process (SDC) */
  ACMD41 = 41,
  /** Leading command for ACMD* */
  CMD55 = 55,
  /** Read OCR */
  CMD58 = 58,
  /** Enable/disable CRC check */
  CMD59 = 59,
}

#[derive(Debug, Copy, Clone)]
pub enum InitError {
  CMDFailed(CMD, u8),
  CardCapacityStatusNotSet([u8; 4]),
  CannotGetCardInfo,
}

/**
 * Card Specific Data: CSD Register
 */
#[derive(Debug, Copy, Clone)]
pub struct SD_CSD {
  pub CSDStruct: u8,        /* CSD structure */
  pub SysSpecVersion: u8,   /* System specification version */
  pub Reserved1: u8,        /* Reserved */
  pub TAAC: u8,             /* Data read access-time 1 */
  pub NSAC: u8,             /* Data read access-time 2 in CLK cycles */
  pub MaxBusClkFrec: u8,    /* Max. bus clock frequency */
  pub CardComdClasses: u16, /* Card command classes */
  pub RdBlockLen: u8,       /* Max. read data block length */
  pub PartBlockRead: u8,    /* Partial blocks for read allowed */
  pub WrBlockMisalign: u8,  /* Write block misalignment */
  pub RdBlockMisalign: u8,  /* Read block misalignment */
  pub DSRImpl: u8,          /* DSR implemented */
  pub Reserved2: u8,        /* Reserved */
  pub DeviceSize: u32,      /* Device Size */
  //MaxRdCurrentVDDMin: u8,   /* Max. read current @ VDD min */
  //MaxRdCurrentVDDMax: u8,   /* Max. read current @ VDD max */
  //MaxWrCurrentVDDMin: u8,   /* Max. write current @ VDD min */
  //MaxWrCurrentVDDMax: u8,   /* Max. write current @ VDD max */
  //DeviceSizeMul: u8,        /* Device size multiplier */
  pub EraseGrSize: u8,         /* Erase group size */
  pub EraseGrMul: u8,          /* Erase group size multiplier */
  pub WrProtectGrSize: u8,     /* Write protect group size */
  pub WrProtectGrEnable: u8,   /* Write protect group enable */
  pub ManDeflECC: u8,          /* Manufacturer default ECC */
  pub WrSpeedFact: u8,         /* Write speed factor */
  pub MaxWrBlockLen: u8,       /* Max. write data block length */
  pub WriteBlockPaPartial: u8, /* Partial blocks for write allowed */
  pub Reserved3: u8,           /* Reserded */
  pub ContentProtectAppli: u8, /* Content protection application */
  pub FileFormatGroup: u8,     /* File format group */
  pub CopyFlag: u8,            /* Copy flag (OTP) */
  pub PermWrProtect: u8,       /* Permanent write protection */
  pub TempWrProtect: u8,       /* Temporary write protection */
  pub FileFormat: u8,          /* File Format */
  pub ECC: u8,                 /* ECC code */
  pub CSD_CRC: u8,             /* CSD CRC */
  pub Reserved4: u8,           /* always 1*/
}

/**
 * Card Identification Data: CID Register
 */
#[derive(Debug, Copy, Clone)]
pub struct SD_CID {
  pub ManufacturerID: u8, /* ManufacturerID */
  pub OEM_AppliID: u16,   /* OEM/Application ID */
  pub ProdName1: u32,     /* Product Name part1 */
  pub ProdName2: u8,      /* Product Name part2*/
  pub ProdRev: u8,        /* Product Revision */
  pub ProdSN: u32,        /* Product Serial Number */
  pub Reserved1: u8,      /* Reserved1 */
  pub ManufactDate: u16,  /* Manufacturing Date */
  pub CID_CRC: u8,        /* CID CRC */
  pub Reserved2: u8,      /* always 1 */
}


const GPIOHS_MMIO_BASE: usize = 0x8_0000_0000 + 0x3800_1000;
const SPI0_MMIO_BASE: usize =   0x8_0000_0000 + 0x5200_0000;
const DMAC_MMIO_BASE: usize =   0x8_0000_0000 + 0x5000_0000;
const SYSCTL_MMIO_BASE: usize = 0x8_0000_0000 + 0x5044_0000;

fn gpiohs_set_pin(pin: u8, value: bool) {
  unsafe {
    let output_val = (GPIOHS_MMIO_BASE + 0x0c) as *mut u32;
    let old_value = output_val.read_volatile();
    if value {
      output_val.write_volatile(old_value | (1 << pin));
    } else {
      output_val.write_volatile(old_value & !(1 << pin));
    }
  }
}


fn spi_configure(
  work_mode: u8, // 0,1,2,3
  frame_format: u8, // 0 standard, 1 dual, 2 quad, 3 octal 
  data_bit_length: u8,
  endian: u32,
  instruction_length: u8,
  address_length: u8,
  wait_cycles: u8,
  instruction_address_trans_mode: u8, // 0 standard, 1 addr standard, 2 as frame format
  tmod: u8, // 0 trans recv, 1 trans, 2 recv, 3 eerom
) {
  assert!(data_bit_length >= 4 && data_bit_length <= 32);
  assert!(wait_cycles < (1 << 5));
  let inst_l: u8 = match instruction_length {
    0 => 0,
    4 => 1,
    8 => 2,
    16 => 3,
    _ => panic!("unhandled intruction length"),
  };

  assert!(address_length % 4 == 0 && address_length <= 60);
  let addr_l: u8 = address_length / 4;
  
  let imr = (SPI0_MMIO_BASE + 0x2c) as *mut u32;
  let dmacr = (SPI0_MMIO_BASE + 0x4c) as *mut u32;
  let dmatdlr = (SPI0_MMIO_BASE + 0x50) as *mut u32;
  let dmardlr = (SPI0_MMIO_BASE + 0x54) as *mut u32;
  let ser = (SPI0_MMIO_BASE + 0x10) as *mut u32;
  let ssienr = (SPI0_MMIO_BASE + 0x08) as *mut u32;
  let ctrlr0 = (SPI0_MMIO_BASE + 0x00) as *mut u32;
  let spi_ctrlr0 = (SPI0_MMIO_BASE + 0xf4) as *mut u32;
  let endian = (SPI0_MMIO_BASE + 0x118) as *mut u32;

  unsafe {
    imr.write_volatile(0x00);
    dmacr.write_volatile(0x00);
    dmatdlr.write_volatile(0x10);
    dmardlr.write_volatile(0x00);
    ser.write_volatile(0x00);
    ssienr.write(0x00);
    // work_mode << 6 (2)
    // tmod << 8 (2)
    // frame_format << 21 (2)
    // data_length << 16 (5)
    ctrlr0.write_volatile(
      (work_mode as u32) << 6
        | (tmod as u32) << 8
        | (frame_format as u32) << 21
        | (data_bit_length as u32 - 1) << 16
    );
    // aitm << 0 (2)
    // addr_length << 2 (4)
    // instruction_length << 8 (2)
    // wait_cycles << 11 (5)
    spi_ctrlr0.write_volatile(
      (instruction_address_trans_mode as u32) << 0
        | (addr_l as u32) << 2
        | (inst_l as u32) << 8
        | (wait_cycles as u32) << 11
    );
    endian.write_volatile(endian as u32);
  }
}

fn spi_send_data(chip_select: u32, tx: &[u8]) {
  let ser = (SPI0_MMIO_BASE + 0x10) as *mut u32;
  let ssienr = (SPI0_MMIO_BASE + 0x08) as *mut u32;
  let txflr = (SPI0_MMIO_BASE + 0x20) as *mut u32;
  let dr0 = (SPI0_MMIO_BASE + 0x60) as *mut u32;
  let sr = (SPI0_MMIO_BASE + 0x28) as *mut u32;
  unsafe {
    ser.write_volatile(1 << chip_select);
    ssienr.write_volatile(0x01);

    let mut fifo_len = 0;
    for &val in tx {
      while fifo_len == 0 {
        fifo_len = 32 - txflr.read_volatile();
      }
      dr0.write_volatile(val as u32);
      fifo_len -= 1;
    }

    while (sr.read_volatile() & 0x05) != 0x04 {
      // IDLE
    }
    ser.write_volatile(0x00);
    ssienr.write_volatile(0x00);
  }
}

fn dmac_set_channel_param(channel_num: u8,
                          src: u64, dest: u64, src_inc: u8, dest_inc: u8,
                          burst_size: u8,
                          trans_width: u8,
                          block_size: u32) {
  fn is_memory(address: u64) -> bool {
    let mem_len = 6 * 1024 * 1024;
    let mem_no_cache_len = 8 * 1024 * 1024;
    // Note: This comes from the Kendryte SDK as-is. No, I have no idea why the AES accelerator
    // input address 0x50450040 is considered memory, either.
    ((address >= 0x80000000) && (address < 0x80000000 + mem_len))
      || ((address >= 0x40000000) && (address < 0x40000000 + mem_no_cache_len))
      || (address == 0x50450040)
  }
  let src_is_mem = is_memory(src);
  let dest_is_mem = is_memory(dest);
  let flow_control: u64 = match (src_is_mem, dest_is_mem) {
    (false, false) => 3,
    (true, false) => 1,
    (false, true) => 2,
    (true, true) => 0,
  };

  let sar = (DMAC_MMIO_BASE + 0x100) as *mut u64;
  let dar = (DMAC_MMIO_BASE + 0x108) as *mut u64;
  let block_ts = (DMAC_MMIO_BASE + 0x110) as *mut u64;
  let ctl = (DMAC_MMIO_BASE + 0x118) as *mut u64;
  let cfg = (DMAC_MMIO_BASE + 0x120) as *mut u64;
  unsafe {
    // cfg TODO: modify
    //  - tt_fc 32:34
    //  - hs_sel_src 35
    //  - hs_sel_dst 36
    //  - src_per 39:42
    //  - dst_per 44:47
    //  - src_multblk_type 0:1
    //  - dst_multblk_type 2:3
    cfg.write_volatile(
      flow_control << 32
        | (if src_is_mem { 1 } else { 0 } ) << 35
        | (if dest_is_mem { 1 } else { 0 } ) << 36
        | (channel_num as u64) << 39
        | (channel_num as u64) << 44
    );
    sar.write_volatile(src);
    dar.write_volatile(dest);
    // ctl TODO: modify
    // - sms 0
    // - dms 2
    // - sinc 4
    // - dinc 6
    // - src_tr_width 8:10
    // - dst_tr_width 11:13
    // - src_msize 14:17
    // - dst_msize 18:21
    ctl.write_volatile(
      1 << 2
        | (src_inc as u64) << 4
        | (dest_inc as u64) << 6
        | (trans_width as u64) << 8
        | (trans_width as u64) << 11
        | (burst_size as u64) << 14
        | (burst_size as u64) << 18
    );

    block_ts.write_volatile((block_size - 1) as u64);
  }
}

fn dmac_set_single_mode(channel_num: u8,
                        src: u64, dest: u64, src_inc: u8, // increment 0
                        dest_inc: u8, // nochange 1
                        burst_size: u8, // length_4 1
                        trans_width: u8, // width_32 2
                        block_size: u32) {
  let dmac_channel_0_int_clear = (DMAC_MMIO_BASE + 0x198) as *mut u64;
  let dmac_chen = (DMAC_MMIO_BASE + 0x18) as *mut u64;
  let dmac_cfg = (DMAC_MMIO_BASE + 0x10) as *mut u64;
  unsafe {
    // channel_interrupt_clear
    dmac_channel_0_int_clear.write_volatile(0xffffffff);
    // channel_disable
    dmac_chen.write_volatile(1 << 8);
    // wait_idle
    while (dmac_chen.read_volatile() & 1) == 1 {}
    dmac_channel_0_int_clear.write_volatile(0xffffffff);
    dmac_set_channel_param(channel_num, src, dest, src_inc, dest_inc,
                      burst_size, trans_width, block_size);
    // enable
    dmac_cfg.write_volatile(0b11);
    // channel enable
    dmac_chen.write_volatile(1 << 8 | 1);
  }
}

fn spi_send_data_dma(channel_num: u8, chip_select: u32, tx: &[u32]) {
  let ser = (SPI0_MMIO_BASE + 0x10) as *mut u32;
  let dmacr = (SPI0_MMIO_BASE + 0x4c) as *mut u32;
  let ssienr = (SPI0_MMIO_BASE + 0x08) as *mut u32;
  let dr0 = (SPI0_MMIO_BASE + 0x60) as *mut u32;
  let sr = (SPI0_MMIO_BASE + 0x28) as *mut u32;
  let sysctl_dma_sel0 = (SYSCTL_MMIO_BASE + 0x64) as *mut u32;
  let dmac_chen = (DMAC_MMIO_BASE + 0x18) as *mut u64;
  let dmac_channel_0_int_clear = (DMAC_MMIO_BASE + 0x198) as *mut u64;
  unsafe {
    dmacr.write_volatile(0x2);
    ssienr.write_volatile(0x01);

    // dma_select
    sysctl_dma_sel0.write_volatile(1);
    dmac_set_single_mode(channel_num, virt_to_phys(tx.as_ptr() as usize) as u64, dr0 as u64,
                         0, 1,
                         1, 2, tx.len() as u32);
    ser.write_volatile(1 << chip_select);
    // dmac_wait_done
    while (dmac_chen.read_volatile() & 1) == 1 {}
    dmac_channel_0_int_clear.write_volatile(0xffffffff);

    while (sr.read_volatile() & 0x05) != 0x04 {
      // IDLE
    }
    ser.write_volatile(0x00);
    ssienr.write_volatile(0x00);
  }
}


fn spi_recv_data(chip_select: u32, rx: &mut [u8]) {
  let ctrlr1 = (SPI0_MMIO_BASE + 0x04) as *mut u32;
  let ser = (SPI0_MMIO_BASE + 0x10) as *mut u32;
  let ssienr = (SPI0_MMIO_BASE + 0x08) as *mut u32;
  let rxflr = (SPI0_MMIO_BASE + 0x24) as *mut u32;
  let dr0 = (SPI0_MMIO_BASE + 0x60) as *mut u32;

  if rx.len() == 0 {
    return;
  }
  unsafe {
    ctrlr1.write_volatile((rx.len() - 1) as u32);
    ssienr.write_volatile(0x01);
    dr0.write_volatile(0xffffffff);
    ser.write_volatile(1 << chip_select);

    let mut fifo_len = 0;
    for val in rx.iter_mut() {
      while fifo_len == 0 {
        fifo_len = rxflr.read_volatile();
      }
      *val = dr0.read_volatile() as u8;
      fifo_len -= 1;
    }

    ser.write_volatile(0x00);
    ssienr.write_volatile(0x00);
  }
}

fn spi_recv_data_dma(channel_num: u8, chip_select: u32, rx: &mut [u32]) {
  if rx.len() == 0 {
    return;
  }
  let ctrlr1 = (SPI0_MMIO_BASE + 0x04) as *mut u32;
  let ssienr = (SPI0_MMIO_BASE + 0x08) as *mut u32;
  let dmacr = (SPI0_MMIO_BASE + 0x4c) as *mut u32;
  let dr0 = (SPI0_MMIO_BASE + 0x60) as *mut u32;
  let ser = (SPI0_MMIO_BASE + 0x10) as *mut u32;
  let sysctl_dma_sel0 = (SYSCTL_MMIO_BASE + 0x64) as *mut u32;
  let dmac_chen = (DMAC_MMIO_BASE + 0x18) as *mut u64;
  let dmac_channel_0_int_clear = (DMAC_MMIO_BASE + 0x198) as *mut u64;

  unsafe {
    ctrlr1.write_volatile((rx.len() - 1) as u32);
    ssienr.write_volatile(0x01);
    dmacr.write_volatile(0x3);
    // dma_select
    sysctl_dma_sel0.write_volatile(0);
    dmac_set_single_mode(channel_num, dr0 as u64,virt_to_phys(rx.as_ptr() as usize) as u64,
                         1, 0,
                         0, 2, rx.len() as u32);
    dr0.write_volatile(0xffffffff);
    ser.write_volatile(1 << chip_select);
    // dmac_wait_done
    while (dmac_chen.read_volatile() & 1) == 1 {}
    dmac_channel_0_int_clear.write_volatile(0xffffffff);

    ser.write_volatile(0x00);
    ssienr.write_volatile(0x00);
  }
}

const SD_CS: u32 = 3;
const SD_CS_GPIONUM: u8 = 7;

/**
 * Card information
 */
#[derive(Debug, Copy, Clone)]
pub struct SD_CardInfo {
  pub SD_csd: SD_CSD,
  pub SD_cid: SD_CID,
  pub CardCapacity: u64,  /* Card Capacity */
  pub CardBlockSize: u64, /* Card Block Size */
}

impl SDCard {

  fn CS_HIGH(&self) {
    gpiohs_set_pin(SD_CS_GPIONUM, true);
  }

  fn CS_LOW(&self) {
    gpiohs_set_pin(SD_CS_GPIONUM, false);
  }

  fn write_data(&self, data: &[u8]) {
    spi_configure(
      0,
      0,
      8, /* data bits */
      0, /* endian */
      0, /*instruction length*/
      0, /*address length*/
      0, /*wait cycles*/
      0,
      1,
    );
    spi_send_data(SD_CS, data);
  }

  fn write_data_dma(&self, data: &[u32]) {
    spi_configure(
      0,
      0,
      8, /* data bits */
      0, /* endian */
      0, /*instruction length*/
      0, /*address length*/
      0, /*wait cycles*/
      0,
      1,
    );
    spi_send_data_dma(0, SD_CS, data);
  }

  fn read_data(&self, data: &mut [u8]) {
    spi_configure(
      0,
      0,
      8, /* data bits */
      0, /* endian */
      0, /*instruction length*/
      0, /*address length*/
      0, /*wait cycles*/
      0,
      2,
    );
    spi_recv_data(SD_CS, data);
  }

  fn read_data_dma(&self, data: &mut [u32]) {
    spi_configure(
      0,
      0,
      8, /* data bits */
      0, /* endian */
      0, /*instruction length*/
      0, /*address length*/
      0, /*wait cycles*/
      0,
      2,
    );
    spi_recv_data_dma(0, SD_CS, data);
  }

  /*
   * Send 5 bytes command to the SD card.
   * @param  cmd: The user expected command to send to SD card.
   * @param  arg: The command argument.
   * @param  crc: The CRC.
   * @retval None
   */
  fn send_cmd(&self, cmd: CMD, arg: u32, crc: u8) {
    /* SD chip select low */
    self.CS_LOW();
    /* Send the Cmd bytes */
    self.write_data(&[
      /* Construct byte 1 */
      ((cmd as u8) | 0x40),
      /* Construct byte 2 */
      (arg >> 24) as u8,
      /* Construct byte 3 */
      ((arg >> 16) & 0xff) as u8,
      /* Construct byte 4 */
      ((arg >> 8) & 0xff) as u8,
      /* Construct byte 5 */
      (arg & 0xff) as u8,
      /* Construct CRC: byte 6 */
      crc,
    ]);
  }

  /* Send end-command sequence to SD card */
  fn end_cmd(&self) {
    /* SD chip select high */
    self.CS_HIGH();
    /* Send the cmd byte */
    self.write_data(&[0xff]);
  }

  /*
   * Returns the SD response.
   * @param  None
   * @retval The SD Response:
   *         - 0xFF: Sequence failed
   *         - 0: Sequence succeed
   */
  fn get_response(&self) -> u8 {
    let result = &mut [0u8];
    let mut timeout = 0x0FFF;
    /* Check if response is got or a timeout is happen */
    while timeout != 0 {
      self.read_data(result);
      /* Right response got */
      if result[0] != 0xFF {
        return result[0];
      }
      timeout -= 1;
    }
    /* After time out */
    return 0xFF;
  }

  /*
   * Get SD card data response.
   * @param  None
   * @retval The SD status: Read data response xxx0<status>1
   *         - status 010: Data accecpted
   *         - status 101: Data rejected due to a crc error
   *         - status 110: Data rejected due to a Write error.
   *         - status 111: Data rejected due to other error.
   */
  fn get_dataresponse(&self) -> u8 {
    let response = &mut [0u8];
    /* Read resonse */
    self.read_data(response);
    /* Mask unused bits */
    response[0] &= 0x1F;
    if response[0] != 0x05 {
      return 0xFF;
    }
    /* Wait null data */
    self.read_data(response);
    while response[0] == 0 {
      self.read_data(response);
    }
    /* Return response */
    return 0;
  }

  /*
   * Reads a block of data from the SD.
   * @param  data_buf: slice that receives the data read from the SD.
   * @param  sector: SD's internal address to read from.
   * @retval The SD Response:
   *         - `Err(())`: Sequence failed
   *         - `Ok(())`: Sequence succeed
   */
  pub fn read_sector(&self, data_buf: &mut [u8], sector: u32) -> Result<(), ()> {
    assert!(data_buf.len() >= SEC_LEN && (data_buf.len() % SEC_LEN) == 0);
    /* Send CMD17 to read one block, or CMD18 for multiple */
    let flag = if data_buf.len() == SEC_LEN {
      self.send_cmd(CMD::CMD17, sector, 0);
      false
    } else {
      self.send_cmd(CMD::CMD18, sector, 0);
      true
    };
    /* Check if the SD acknowledged the read block command: R1 response (0x00: no errors) */
    if self.get_response() != 0x00 {
      self.end_cmd();
      return Err(());
    }
    let mut error = false;
    let mut dma_chunk = [0u32; SEC_LEN];
    for chunk in data_buf.chunks_mut(SEC_LEN) {
      if self.get_response() != SD_START_DATA_SINGLE_BLOCK_READ {
        error = true;
        break;
      }
      /* Read the SD block data : read NumByteToRead data */
      self.read_data_dma(&mut dma_chunk);
      /* Place the data received as u32 units from DMA into the u8 target buffer */
      for (a, b) in chunk.iter_mut().zip(dma_chunk.iter()) {
        *a = (b & 0xff) as u8;
      }
      /* Get CRC bytes (not really needed by us, but required by SD) */
      let mut frame = [0u8; 2];
      self.read_data(&mut frame);
    }
    self.end_cmd();
    if flag {
      self.send_cmd(CMD::CMD12, 0, 0);
      self.get_response();
      self.end_cmd();
      self.end_cmd();
    }
    /* It is an error if not everything requested was read */
    if error {
      Err(())
    } else {
      Ok(())
    }
  }

  /*
   * Writes a block to the SD
   * @param  data_buf: slice containing the data to be written to the SD.
   * @param  sector: address to write on.
   * @retval The SD Response:
   *         - `Err(())`: Sequence failed
   *         - `Ok(())`: Sequence succeed
   */
  pub fn write_sector(&self, data_buf: &[u8], sector: u32) -> Result<(), ()> {
    assert!(data_buf.len() >= SEC_LEN && (data_buf.len() % SEC_LEN) == 0);
    let mut frame = [0xff, 0x00];
    if data_buf.len() == SEC_LEN {
      frame[1] = SD_START_DATA_SINGLE_BLOCK_WRITE;
      self.send_cmd(CMD::CMD24, sector, 0);
    } else {
      frame[1] = SD_START_DATA_MULTIPLE_BLOCK_WRITE;
      self.send_cmd(
        CMD::ACMD23,
        (data_buf.len() / SEC_LEN).try_into().unwrap(),
        0,
      );
      self.get_response();
      self.end_cmd();
      self.send_cmd(CMD::CMD25, sector, 0);
    }
    /* Check if the SD acknowledged the write block command: R1 response (0x00: no errors) */
    if self.get_response() != 0x00 {
      self.end_cmd();
      return Err(());
    }
    let mut dma_chunk = [0u32; SEC_LEN];
    for chunk in data_buf.chunks(SEC_LEN) {
      /* Send the data token to signify the start of the data */
      self.write_data(&frame);
      /* Write the block data to SD : write count data by block */
      for (a, &b) in dma_chunk.iter_mut().zip(chunk.iter()) {
        *a = b.into();
      }
      self.write_data_dma(&mut dma_chunk);
      /* Put dummy CRC bytes */
      self.write_data(&[0xff, 0xff]);
      /* Read data response */
      if self.get_dataresponse() != 0x00 {
        self.end_cmd();
        return Err(());
      }
    }
    self.end_cmd();
    self.end_cmd();
    Ok(())
  }
}

pub fn server() {
  let sdcard = SDCard;
  let buf = valloc(1);
  unsafe { sdcard.read_sector(core::slice::from_raw_parts_mut(buf as *mut u8, 4096), 8); }
  info!("read ok");
  let content = unsafe { core::slice::from_raw_parts(buf, 4096) };
  for i in 0..4096 {
    print!("{:02x} ", content[i]);
    if (i + 1) % 16 == 0 {
      println!();
    }
  }
  loop {}
}
