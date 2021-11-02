ENTRY(_start)

SECTIONS
{
    . = 0xFFFFFFFF80020000;
    KERNEL_ENTRY = .;
    .init : {
        *(.text.start)
        *(.data.start)
    }
    .text : {
        *(.text*)
    }
    .rodata : {
        *(.rodata*)
    }
    .data : {
        *(.data*)
        *(.sdata*)
    }
    . = ALIGN(4096);
    BSS_START = .;
    .bss : {
        *(.bss.ipc)
        *(.bss*)
        *(.sbss*)
    }
    BSS_END = .;

    KERNEL_END = .;

    ELF_IMAGE = 0xFFFFFFFFc0000000;
}
