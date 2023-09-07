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

        . = ALIGN(8);
        PROVIDE(_sdev = .);
        *(.dev*)
        PROVIDE(_edev = .);
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

        . = ALIGN(PAGE_SIZE);
        PROVIDE(_sstack = .);
        *(.stack)
        PROVIDE(_estack = .);
    }
    PROVIDE(_ebss = .);

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
