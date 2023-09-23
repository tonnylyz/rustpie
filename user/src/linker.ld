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
        *(.sdata*)
        *(.data*)
    }
    . = ALIGN(4096);
    .bss : {
        *(.sbss*)
        *(.bss*)
    }
    /DISCARD/ : {
        *(.eh_frame*)
        *(.gcc_except_table*)
    }
}
