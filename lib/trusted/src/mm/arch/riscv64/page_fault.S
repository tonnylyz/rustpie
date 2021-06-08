.global asm_page_fault_handler
asm_page_fault_handler:
    jal page_fault_handler

    ld t0, 33 * 8(sp) // pc
    ld t1, 2 * 8(sp) // sp
    sd t0, -16(t1)

    ld x1, 1 * 8(sp)
    // no x2(sp) here
    ld x3, 3 * 8(sp)
    //ld x4, 4 * 8(sp) // x4 thread pointer used as reserved
    ld x5, 5 * 8(sp)
    ld x6, 6 * 8(sp)
    ld x7, 7 * 8(sp)
    ld x8, 8 * 8(sp)
    ld x9, 9 * 8(sp)
    ld x10, 10 * 8(sp)
    ld x11, 11 * 8(sp)
    ld x12, 12 * 8(sp)
    ld x13, 13 * 8(sp)
    ld x14, 14 * 8(sp)
    ld x15, 15 * 8(sp)
    ld x16, 16 * 8(sp)
    ld x17, 17 * 8(sp)
    ld x18, 18 * 8(sp)
    ld x19, 19 * 8(sp)
    ld x20, 20 * 8(sp)
    ld x21, 21 * 8(sp)
    ld x22, 22 * 8(sp)
    ld x23, 23 * 8(sp)
    ld x24, 24 * 8(sp)
    ld x25, 25 * 8(sp)
    ld x26, 26 * 8(sp)
    ld x27, 27 * 8(sp)
    ld x28, 28 * 8(sp)
    ld x29, 29 * 8(sp)
    ld x30, 30 * 8(sp)
    ld x31, 31 * 8(sp)

    ld x4, 2 * 8(sp) // sp
    mv sp, x4

    ld x4, -16(sp) // pc
    jr x4

