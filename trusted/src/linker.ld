ENTRY(_start)

SECTIONS
{
    . = 0x40000;
    .text : {
        *(.text*)
    }
    . = ALIGN(4096);
    .rodata : {
        *(.rodata*)
    }
    . = ALIGN(4096);
    .data : {
        *(.data*)
    }
    . = ALIGN(4096);
    .bss : {
        *(.bss*)
    }

    ELF_IMAGE = 0x80000000;
}
