OUTPUT_ARCH(riscv)

ENTRY(_start)

BASE_ADDRESS = 0x80200000;
PAGE_SIZE = 4096;

SECTIONS {
    . = BASE_ADDRESS;

    # Text section

    . = ALIGN(PAGE_SIZE);
    PROVIDE(_stext = .);
    .text : {
        *(.boot)
        *(.text .text*)
    }
    PROVIDE(_etext = .);

    # Data section

    . = ALIGN(PAGE_SIZE);
    PROVIDE(_srodata = .);
    .rodata : {
        *(.rodata .rodata* .srodata .srodata*)
    }
    PROVIDE(_erodata = .);

    . = ALIGN(PAGE_SIZE);
    PROVIDE(_sdata = .);
    .data : {
        *(.data .data* .sdata .sdata*)
    }
    PROVIDE(_edata = .);

    # BSS section

    . = ALIGN(PAGE_SIZE);
    PROVIDE(_sbss = .);
    .bss : {
        *(.bss .bss* .sbss .sbss*)
    }
    PROVIDE(_ebss = .);

    PROVIDE(_end = .);
}
