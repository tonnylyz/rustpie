ENTRY(_start)

SECTIONS
{
    . = 0xFFFF800001800000;
    KERNEL_ENTRY = .;
    .init : {
        *(.text.start)
    }
    . = ALIGN(4096);
    .sec_init : {
        *(.text.sec_start)
    }
    .text : {
        *(.text*)
    }
    .rodata : {
        *(.rodata*)
    }
    .data : {
        *(.data*)
    }
    . = ALIGN(4096);
    BSS_START = .;
    .bss : {
        *(.bss*)
    }
    . = ALIGN(4096);
    BSS_END = .;
    .stack (NOLOAD) : {
        *(.stack)
    }
    KERNEL_END = .;
}
