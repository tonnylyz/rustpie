pub mod vm_descriptor {
  use tock_registers::register_bitfields;

  register_bitfields! {u64,
    pub TABLE_DESCRIPTOR [
      NEXT_LEVEL_TABLE_PPN OFFSET(12) NUMBITS(36) [], // [47:12]
      TYPE  OFFSET(1) NUMBITS(1) [
        Block = 0,
        Table = 1
      ],
      VALID OFFSET(0) NUMBITS(1) [
        False = 0,
        True = 1
      ]
    ]
  }

  register_bitfields! {u64,
    pub PAGE_DESCRIPTOR [
      // Note: LIB and COW are software-defined bits
      LIB      OFFSET(56) NUMBITS(1) [
        False = 0,
        True = 1
      ],
      COW      OFFSET(55) NUMBITS(1) [
        False = 0,
        True = 1
      ],
      UXN      OFFSET(54) NUMBITS(1) [
        False = 0,
        True = 1
      ],
      PXN      OFFSET(53) NUMBITS(1) [
        False = 0,
        True = 1
      ],
      OUTPUT_PPN OFFSET(12) NUMBITS(36) [], // [47:12]
      AF       OFFSET(10) NUMBITS(1) [
        False = 0,
        True = 1
      ],
      SH       OFFSET(8) NUMBITS(2) [
        OuterShareable = 0b10,
        InnerShareable = 0b11
      ],
      AP       OFFSET(6) NUMBITS(2) [
        RW_EL1 = 0b00,
        RW_EL1_EL0 = 0b01,
        RO_EL1 = 0b10,
        RO_EL1_EL0 = 0b11
      ],
      AttrIndx OFFSET(2) NUMBITS(3) [
        NORMAL = 0b000,
        DEVICE = 0b001
      ],
      TYPE     OFFSET(1) NUMBITS(1) [
        Block = 0,
        Table = 1
      ],
      VALID    OFFSET(0) NUMBITS(1) [
        False = 0,
        True = 1
      ]
    ]
  }
}