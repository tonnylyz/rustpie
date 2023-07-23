ENTRY(_start)

SECTIONS
{
    . = 4096;
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
    .sdata : {
        *(.sdata*)
    }

    . = ALIGN(4096);
    .bss : {
        *(.bss*)
        *(.sbss*)
    }
    .sbss : {
        *(.sbss*)
    }

    ELF_IMAGE = 0x80000000;
}
