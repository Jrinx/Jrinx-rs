OUTPUT_ARCH(%ARCH%)

ENTRY(_start)

CACHE_LINE_SIZE = 64;
PAGE_SIZE = 4096;

SECTIONS {
    . = %BASE_ADDRESS%;

    # Text section

    . = ALIGN(PAGE_SIZE);
    .text : {
        PROVIDE(_stext = .);
        *(.boot)
        *(.text .text*)
        PROVIDE(_etext = .);
    }

    # Data section

    . = ALIGN(PAGE_SIZE);
    .rodata : {
        PROVIDE(_srodata = .);
        *(.rodata .rodata* .srodata .srodata*)

        . = ALIGN(8);
        PROVIDE(_sdev = .);
        *(.dev.device_type*)
        *(.dev.compatible*)
        *(.dev*)
        PROVIDE(_edev = .);

        . = ALIGN(8);
        PROVIDE(_stest = .);
        *(.test*)
        PROVIDE(_etest = .);
        PROVIDE(_erodata = .);
    }

    . = ALIGN(PAGE_SIZE);
    .data : {
        PROVIDE(_sdata = .);
        *(.data .data* .sdata .sdata*)
        PROVIDE(_edata = .);
    }

    . = ALIGN(PAGE_SIZE);
    .percpu : {
        PROVIDE(_spercpu = .);
        *(.percpu .percpu*)
        . = ALIGN(CACHE_LINE_SIZE);
        PROVIDE(_epercpu = .);
    }

    # BSS section

    . = ALIGN(PAGE_SIZE);
    .bss : {
        PROVIDE(_sbss = .);
        *(.bss .bss* .sbss .sbss*)
        PROVIDE(_ebss = .);

        . = ALIGN(PAGE_SIZE);
        PROVIDE(_sstack = .);
        *(.stack)
        PROVIDE(_estack = .);
    }

    # Discarded sections

    /DISCARD/ : {
        *(.comment)
        *(.eh_frame*)
        *(.note*)
        *(.gnu*)
    }

    # End of memory

    PROVIDE(_end = .);
}
