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
    .eh_frame_hdr : { *(.eh_frame_hdr) }
    .eh_frame : { KEEP (*(.eh_frame)) }
    .gcc_except_table : { *(.gcc_except_table) }

    . = ALIGN(4096);
    .data : {
        *(.data*)
    }
    .got : { *(.got.plt) *(.got) }
    . = ALIGN(4096);
    .bss : {
        *(.bss*)
    }

    ELF_IMAGE = 0x80000000;
}
