SECTIONS
{
    . = 0xFFFFFF8040080000;
    .init : {
        *(.text.start)
        *(.data.start)
        *(.text.kvm)
        *(.data.kvm)
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
}
