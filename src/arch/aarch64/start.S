.section .text.start

.global _start
_start:
    mrs x1, mpidr_el1
#    and x2, x1, #0x100
#    cbnz x2, _loop
    and x1, x1, #0xff
    mrs x2, currentel
    cmp x2, #0x4
    beq _el1_entry
    cmp x2, #0x8
    beq _el2_entry
    b _loop

_el2_entry:
    ldr x3, =(1 << 31) // EL1 is aarch64
    msr hcr_el2, x3

    ldr x3, =((3 << 28) | (3 << 22) | (1 << 20) | (1 << 11)) // Reserved-1
    msr sctlr_el1, x3

    ldr x3, =((7 << 6) | 5) // El1h mask all
    msr spsr_el2, x3

    adr x3, _el1_entry
    msr elr_el2, x3
    eret

_el1_entry:
    mov x19, x1
    mov x1, #1
    msr spsel, x1
    adrp x0, BOOT_STACK_TOP
    mov x1, 4096
    mul x1, x1, x19
    sub x0, x0, x1
    mov sp, x0
    cbnz x19, 1f
    adrp x0, BSS_START
    adrp x1, BSS_END
    sub  x2, x1, x0
    mov  x1, 0
    bl memset
    adrp x0, KERNEL_PAGE_DIRECTORY
    bl populate_page_table
1:  adrp x0, KERNEL_PAGE_DIRECTORY
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

.data
BOOT_STACK:
.align 12
.space 4096 * 4
BOOT_STACK_TOP:

.align 12
.global KERNEL_PAGE_DIRECTORY
KERNEL_PAGE_DIRECTORY:
    .space 4096