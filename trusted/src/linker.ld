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
    .gcc_except_table : {
        *(.gcc_except_table*)
    }

    . = ALIGN(4096);
    .data : {
        *(.data*)
    }

    . = ALIGN(4096);
    .bss : {
        *(.sbss*)
        *(.bss*)
    }

    ELF_IMAGE = 0x80000000;
}
