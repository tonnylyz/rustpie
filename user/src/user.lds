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
        *(.eh_frame*)
    }
    . = ALIGN(4096);
    .bss : {
        *(.bss*)
        *(.sbss*)
    }
    /DISCARD/ : {
        *(.comment)
    }
}
