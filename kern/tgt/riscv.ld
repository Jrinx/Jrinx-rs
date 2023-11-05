OUTPUT_ARCH(riscv)

ENTRY(_start)
INCLUDE kern/tgt/kernel-vars.ld-inc

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
        *(.dev.device_type*)
        *(.dev.compatible*)
        *(.dev*)
        PROVIDE(_edev = .);

        . = ALIGN(8);
        PROVIDE(_stest = .);
        *(.test*)
        PROVIDE(_etest = .);
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
