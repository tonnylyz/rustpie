// Because low 2 bits of STVEC are used, it requires vector entry to be properly aligned.
.align 4
.global push_context
push_context:
    csrrw sp, sscratch, sp // (sp, sscratch) -> (kernel sp, thread sp)
    addi  sp, sp, -0x110 // size of ContextFrame
    sd x1, 1 * 8(sp)
    //this x2 is kernel sp
    sd x3, 3 * 8(sp)
    sd x4, 4 * 8(sp)
    sd x5, 5 * 8(sp)
    sd x6, 6 * 8(sp)
    sd x7, 7 * 8(sp)
    sd x8, 8 * 8(sp)
    sd x9, 9 * 8(sp)
    sd x10, 10 * 8(sp)
    sd x11, 11 * 8(sp)
    sd x12, 12 * 8(sp)
    sd x13, 13 * 8(sp)
    sd x14, 14 * 8(sp)
    sd x15, 15 * 8(sp)
    sd x16, 16 * 8(sp)
    sd x17, 17 * 8(sp)
    sd x18, 18 * 8(sp)
    sd x19, 19 * 8(sp)
    sd x20, 20 * 8(sp)
    sd x21, 21 * 8(sp)
    sd x22, 22 * 8(sp)
    sd x23, 23 * 8(sp)
    sd x24, 24 * 8(sp)
    sd x25, 25 * 8(sp)
    sd x26, 26 * 8(sp)
    sd x27, 27 * 8(sp)
    sd x28, 28 * 8(sp)
    sd x29, 29 * 8(sp)
    sd x30, 30 * 8(sp)
    sd x31, 31 * 8(sp)

    addi s0, sp, 0x110
    // |core_id| <-- sscratch
    // |-------|
    // |       |
    // |__ctx__| <-- sp
    csrrw s0, sscratch, s0 // s0 <- thread's sp, sscratch <- core_id
    csrr s1, sstatus
    csrr s2, sepc

    sd s0, 2 * 8(sp)
    sd s1, 32 * 8(sp)
    sd s2, 33 * 8(sp)

    mv a0, sp
    mv s0, sp
    jal exception_entry // see exception.rs
    j pop_context


.global pop_context_first // pop_context_first(*ctx, core_id)
pop_context_first:
    // |core_id| <-- sp + 0x110
    // |-------|
    // |       |
    // |__ctx__| <-- sp
    mv sp, a0
    sd a1, 0x110(sp)
pop_context:
    // kernel sp points to bottom of a context frame
    addi s0, sp, 0x110 // size of ContextFrame
    csrw sscratch, s0 // save kernel sp back to sscratch
    ld s1, 32 * 8(sp)
    ld s2, 33 * 8(sp)
    csrw sstatus, s1
    csrw sepc, s2

    ld x1, 1 * 8(sp)
    // no x2(sp) here
    ld x3, 3 * 8(sp)
    ld x4, 4 * 8(sp)
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

    ld x2, 2 * 8(sp)// restore user sp
    sret
