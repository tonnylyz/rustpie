ENTRY(_start)

SECTIONS
{
    . = 0xFFFFFFFF80200000;
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
    BSS_END = .;

    KERNEL_END = .;

    KERNEL_ELF = 0xFFFFFFFFc0000000;
}
