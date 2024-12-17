
ENTRY(start)

SECTIONS {
    . = 0x0000000001000000;

    .text ALIGN(4096): {
        KEEP(*(.text.start));
        *(.text .text.*);
    }

    .rodata ALIGN(4096): {
        *(.rodata .rodata.*);
        . = ALIGN(16);
        *(.srodata .srodata.*);
        KEEP(*(.symbols));
    }

    .data ALIGN(4096):  {
        *(.data .data.*);
        *(.sdata .sdata.*);
    }

    .bss ALIGN(4096):  {
        *(.bss .bss.);
        . = ALIGN(16);
        *(.sbss .sbss.*);

        . = ALIGN(16); /* https://github.com/nuta/operating-system-in-1000-lines/pull/23 */

        . += 128 * 1024;
        __stack_top = .;

        /*ASSERT(. - _VA_OFFSET < 0x1800000, "hoge large executable");*/
    }

}
