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

    adr x3, BOOT_STACK_TOP
    msr sp_el1, x3
    eret


.global _el1_start
_el1_start:
    mov x19, x1
    mov x1, #1
    msr spsel, x1
    adr x0, BOOT_STACK_TOP
    mov x1, 4096
    mul x1, x1, x19
    sub x0, x0, x1
    mov sp, x0
    cbnz x19, 1f
    adr x0, KERNEL_PAGE_DIRECTORY
    bl populate_page_table
1:  adr x0, KERNEL_PAGE_DIRECTORY
    bl mmu_init
    mov x0, x19
    ldr x1, =stack_of_core
    blr x1
    mov sp, x0
    mov x0, x19
    mov fp, sp
    ldr x1, =main
    blr x1

_loop:
    wfe
    b _loop
.section .data.start
BOOT_STACK:
    .align 12
    .space 4096 * 4
BOOT_STACK_TOP:

.align 12
.global KERNEL_PAGE_DIRECTORY
KERNEL_PAGE_DIRECTORY:
    .space 4096