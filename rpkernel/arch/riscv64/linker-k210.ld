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
        *(.bss*)
        *(.sbss*)
    }
    BSS_END = .;
    .stack (NOLOAD) : {
        *(.stack)
    }
    KERNEL_END = .;

    ELF_IMAGE = 0xFFFFFFFFc0000000;
}
