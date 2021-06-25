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
        *(.got)
    }
    . = ALIGN(4096);
    .bss : {
        *(.bss*)
    }
    /DISCARD/ : {
        *(.eh_frame*)
    }
}
