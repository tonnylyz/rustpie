.global asm_page_fault_handler
asm_page_fault_handler:
    bl page_fault_handler

    ldr x0, [sp, #(8 * 32)] // pc
    ldr x1, [sp, #(8 * 33)] // sp
    str x0, [x1, #-16]

    ldr x0, [sp, #(8 * 0)]
    ldr x1, [sp, #(8 * 1)]
    ldr x2, [sp, #(8 * 2)]
    ldr x3, [sp, #(8 * 3)]
    ldr x4, [sp, #(8 * 4)]
    ldr x5, [sp, #(8 * 5)]
    ldr x6, [sp, #(8 * 6)]
    ldr x7, [sp, #(8 * 7)]

    ldr x8,  [sp, #(8 * 8)]
    ldr x9,  [sp, #(8 * 9)]
    ldr x10, [sp, #(8 * 10)]
    ldr x11, [sp, #(8 * 11)]
    ldr x12, [sp, #(8 * 12)]
    ldr x13, [sp, #(8 * 13)]
    ldr x14, [sp, #(8 * 14)]
    ldr x15, [sp, #(8 * 15)]

    ldr x16, [sp, #(8 * 16)]
    ldr x17, [sp, #(8 * 17)]
    ldr x18, [sp, #(8 * 18)] // x18 PR (reserved purpose)
    ldr x19, [sp, #(8 * 19)]
    ldr x20, [sp, #(8 * 20)]
    ldr x21, [sp, #(8 * 21)]
    ldr x22, [sp, #(8 * 22)]
    ldr x23, [sp, #(8 * 23)]

    ldr x24, [sp, #(8 * 24)]
    ldr x25, [sp, #(8 * 25)]
    ldr x26, [sp, #(8 * 26)]
    ldr x27, [sp, #(8 * 27)]
    ldr x28, [sp, #(8 * 28)]
    ldr x29, [sp, #(8 * 29)]
    ldr x30, [sp, #(8 * 30)]

    ldr x18, [sp, #(8 * 33)] // sp
    mov sp, x18

    ldr x18, [sp, #-16] // pc
    br x18
