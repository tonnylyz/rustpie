.section .text.start

.global _start
_start:
    mrs x1, mpidr_el1
    and x2, x1, #0x100
    cbnz x2, _loop
    and x1, x1, #0xff
    mrs x2, currentel
    cmp x2, #0x4
    beq _el1_start
    cmp x2, #0x8
    beq _el2_start
    b _loop

.global _el2_start
_el2_start:
    mrs x3, hcr_el2
    mov x4, #(1 << 31) // EL1 is aarch64
    orr  x3, x3, x4
    msr hcr_el2, x3

    mrs x3, spsr_el2
    mov x4, #(0b0101) // El1h
    orr  x3, x3, x4
    msr spsr_el2, x3

    adr x3, _el1_start
    msr elr_el2, x3

    adr x3, boot_stack_top
    msr sp_el1, x3
    eret


.global _el1_start
_el1_start:
    mov x19, x1
    adr x0, boot_stack_top
    mov sp, x0
    bl populate_page_table
    bl mmu_init
    mov x0, x19
    ldr x1, =change_stack
    blr x1

_loop:
    wfe
    b _loop
.section .data.start
boot_stack:
    .align 12
    .space 4096
boot_stack_top:
