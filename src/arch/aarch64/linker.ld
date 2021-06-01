ENTRY(_start)

SECTIONS
{
    . = 0xFFFFFF8040080000;
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
    }
    . = ALIGN(4096);
    BSS_START = .;
    .bss : {
        *(.bss.ipc)
        *(.bss*)
    }
    . = ALIGN(4096);
    BSS_END = .;

    KERNEL_END = .;

    KERNEL_ELF = 0xFFFFFF8080000000;
}
